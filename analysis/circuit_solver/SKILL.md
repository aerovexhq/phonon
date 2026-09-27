---
name: circuit_solver
description: Deep numerical algorithms for Modified Nodal Analysis (MNA), sparse LU linear algebra, Newton-Raphson continuation methods, and stiff L-stable DAE integrators (TR-BDF2) in Rust.
---

# Numerical Circuit Solver Engine: MNA, Sparse LU & DAE Integration

## 1. Modified Nodal Analysis (MNA) Matrix Formulation

Modified Nodal Analysis formulates circuit equations by combining Kirchhoff's Current Law (KCL) at all non-reference nodes with auxiliary constitutive branch equations for components whose currents cannot be expressed directly as functions of node voltages (voltage sources, inductors, operational amplifiers).

### 1.1 Block Matrix Partitioning
The MNA algebraic-differential system is represented in standard partitioned block form:

$$\begin{bmatrix} \mathbf{G}_{nn} & \mathbf{B}_{nb} \\ \mathbf{C}_{bn} & \mathbf{D}_{bb} \end{bmatrix} \begin{bmatrix} \mathbf{v}_n \\ \mathbf{i}_b \end{bmatrix} + \begin{bmatrix} \mathbf{C}_{nn} & \mathbf{0} \\ \mathbf{0} & \mathbf{L}_{bb} \end{bmatrix} \frac{d}{dt} \begin{bmatrix} \mathbf{v}_n \\ \mathbf{i}_b \end{bmatrix} + \begin{bmatrix} \mathbf{f}_n(\mathbf{v}_n) \\ \mathbf{f}_b(\mathbf{v}_n, \mathbf{i}_b) \end{bmatrix} = \begin{bmatrix} \mathbf{j}_n(t) \\ \mathbf{e}_b(t) \end{bmatrix}$$

Where:
- $\mathbf{G}_{nn} \in \mathbb{R}^{N \times N}$: Conductance matrix of nodal interconnections.
- $\mathbf{B}_{nb} \in \mathbb{R}^{N \times M}$: Topological incidence matrix connecting branch currents to circuit nodes (+1 for current entering node, -1 for current leaving).
- $\mathbf{C}_{bn} \in \mathbb{R}^{M \times N}$: Transpose/coupling matrix mapping node voltages into branch constraints. For ideal independent voltage sources, $\mathbf{C}_{bn} = \mathbf{B}_{nb}^T$.
- $\mathbf{D}_{bb} \in \mathbb{R}^{M \times M}$: Branch-to-branch dependencies (zero for ideal independent voltage sources and inductors; non-zero for current-controlled voltage sources).
- $\mathbf{C}_{nn}$: Linear nodal capacitance matrix.
- $\mathbf{L}_{bb}$: Branch inductance and mutual coupling matrix.
- $\mathbf{f}_n$: Vector of non-linear semiconductor terminal currents entering nodes.
- $\mathbf{j}_n(t), \mathbf{e}_b(t)$: Independent excitation source vectors.

---

## 2. Sparse Matrix Linear Algebra

Circuit matrices exhibit extreme sparsity (typically $>97\%$ zeros for circuits with $N > 100$). Using dense linear solvers results in $O(N^3)$ computational scaling, which cripples performance. Phonon implements a dedicated **Compressed Sparse Column (CSC)** linear solver optimized for circuit topologies.

### 2.1 Markowitz Minimum-Degree Reordering
Before factorization, matrix rows and columns are permuted to minimize fill-in (creation of non-zero entries during elimination):
$$\mathbf{P} \mathbf{A} \mathbf{Q}^T = \mathbf{L} \mathbf{U}$$
At elimination step $k$, the Markowitz product $M_{ij}$ is evaluated for all non-zero candidates $a_{ij}$:
$$M_{ij} = (r_i - 1)(c_j - 1)$$
Where $r_i$ is the number of non-zero entries in row $i$ and $c_j$ is the number in column $j$.

#### Numerical Stability via Threshold Partial Pivoting:
To prevent numerical instability and catastrophic error amplification when dividing by tiny pivots, a candidate pivot $a_{ij}$ must satisfy the threshold stability criterion:
$$|a_{ij}| \ge u \cdot \max_{k} |a_{kj}|, \quad u \in (0.001, 0.1]$$
Phonon selects the candidate minimizing $M_{ij}$ among all elements satisfying the threshold $u$.

### 2.2 Symbolic vs. Numeric Factorization
Circuit matrix sparsity patterns depend solely on circuit topology, which remains static during simulation.
1. **Symbolic Phase (Executed Once)**:
   Computes elimination trees, predicts fill-in non-zero locations, and pre-allocates pointer index buffers.
2. **Numeric Phase (Executed Every Newton Step)**:
   Fills numerical values directly into pre-allocated memory arrays and performs in-place Gaussian elimination without heap reallocations.

---

## 3. Transient Integration: TR-BDF2 Algorithm

### 3.1 Mathematical Derivation
Traditional SPICE defaults to the Trapezoidal rule, which suffers from spurious high-frequency oscillations (ringing) in stiff circuits, or Gear/BDF methods, which artificially damp physical oscillations due to excessive numerical diffusion.

Phonon solves this by standardizing on **TR-BDF2**, a two-stage composite single-step method:

Given the DAE: $\mathbf{M} \mathbf{\dot{x}} = \mathbf{\phi}(\mathbf{x}, t)$, and time step $h$:

#### Stage 1: Trapezoidal Step to $t_{n+\gamma}$ ($\gamma = 2 - \sqrt{2} \approx 0.5857864$):
$$\mathbf{x}_{n+\gamma} - \frac{\gamma h}{2} \mathbf{\dot{x}}_{n+\gamma} = \mathbf{x}_n + \frac{\gamma h}{2} \mathbf{\dot{x}}_n$$
This stage is second-order accurate and advances the solution to the intermediate point.

#### Stage 2: BDF2 Step to $t_{n+1}$:
$$\mathbf{x}_{n+1} - \frac{1-\gamma}{2-\gamma} h \mathbf{\dot{x}}_{n+1} = \frac{1}{\gamma(2-\gamma)} \mathbf{x}_{n+\gamma} - \frac{(1-\gamma)^2}{\gamma(2-\gamma)} \mathbf{x}_n$$

### 3.2 L-Stability Proof
The stability function of TR-BDF2 is:
$$R(z) = \frac{1 + \frac{(1-\gamma)^2}{\gamma(2-\gamma)} z}{\left(1 - \frac{\gamma}{2} z\right)\left(1 - \frac{1-\gamma}{2-\gamma} z\right)}$$
As $z \to -\infty$, $\lim_{z \to -\infty} R(z) = 0$. This confirms TR-BDF2 is **strictly L-stable**, guaranteeing that stiff high-frequency transients decay exponentially without ringing or instability.

### 3.3 Adaptive Local Truncation Error (LTE) Step Control
At each completed step, TR-BDF2 yields an asymptotic estimate of the local truncation error vector $\mathbf{E}$:
$$\mathbf{E}_{n+1} \approx 2 \frac{1 - 2\gamma + \gamma^2}{(2-\gamma)(1-\gamma)} \left[ \mathbf{\dot{x}}_n - \frac{1}{\gamma} \mathbf{\dot{x}}_{n+\gamma} + \frac{1}{1-\gamma} \mathbf{\dot{x}}_{n+1} \right] \cdot h$$
The scalar error norm is evaluated against user relative and absolute tolerances:
$$\text{err} = \sqrt{\frac{1}{N} \sum_{i=1}^{N} \left( \frac{E_{n+1, i}}{\text{reltol} \cdot |x_{n+1, i}| + \text{abstol}} \right)^2 }$$
The next time step $h_{next}$ is computed adaptively:
$$h_{next} = h \cdot \min\left( \text{h\_max\_ratio}, \max\left( \text{h\_min\_ratio}, 0.9 \left( \frac{1}{\text{err}} \right)^{1/3} \right) \right)$$
If $\text{err} > 1.0$, the current step is rejected, and the solver immediately restarts the step with the reduced $h_{next}$.

---

## 4. Non-Linear Convergence & Continuation Strategies

```
            [ Start Newton-Raphson Iteration ]
                           |
                           v
            [ Solve J(x_k) * dx = -Phi(x_k) ]
                           |
                           v
           [ Apply Damping: x_k+1 = x_k + alpha * dx ]
                           |
         +-----------------+-----------------+
         |                                   |
         v (Converged ||dx|| < tol)          v (Failed / Max Iterations)
  [ Step Accepted ]                 [ Activate Continuation ]
                                             |
                                  +----------+----------+
                                  |                     |
                                  v                     v
                           [ G_min Stepping ]    [ Source Stepping ]
```

### 4.1 Convergence Criteria
A Newton-Raphson step is accepted if and only if both voltage and current tolerances are satisfied:
1. **Voltage Residual**: $|v_i^{(k+1)} - v_i^{(k)}| \le \text{reltol} \cdot \max(|v_i^{(k+1)}|, |v_i^{(k)}|) + \text{vntol}$
2. **KCL Current Residual**: $|\Phi_i(\mathbf{x}^{(k+1)})| \le \text{reltol} \cdot \sum |i_{branch}| + \text{abstol}$

### 4.2 Damping via Node Voltage Limiting
To prevent diode and transistor voltages from taking giant leaps into numerical overflow ($e^{V / V_t}$):
```rust
#[inline]
pub fn limit_pn_junction(v_new: f64, v_old: f64, vt: f64, vcrit: f64) -> f64 {
    if v_new > vcrit && (v_new - v_old).abs() > 2.0 * vt {
        if v_old > 0.0 {
            let arg = 1.0 + (v_new - v_old) / vt;
            if arg > 0.0 {
                v_old + vt * arg.ln()
            } else {
                vcrit
            }
        } else {
            vt * (v_new / vt).ln()
        }
    } else {
        v_new
    }
}
```

### 4.3 Continuation Algorithms
1. **$G_{min}$ Stepping**: Injects conductances $G_{min} = 10^{-2}\text{ S}$ between all non-linear nodes and ground, solving iteratively while logarithmically stepping $G_{min} \to 10^{-12}\text{ S}$.
2. **Source Stepping**: Scales independent sources by $\lambda \in [0, 1]$. Starts at $\lambda = 0$ (where zero voltage is a trivial solution), stepping $\lambda \to 1.0$.

---

## 5. Self-Consistent NEGF-Poisson Electrostatic Solver

In molecular wires, nanoscale Gate-All-Around (GAA) channels, and 2D materials, electron density $\rho$ shifts the electrostatic potential profile $V(\mathbf{r})$, which in turn modifies the tight-binding Hamiltonian $\mathbf{H}_M(V)$.

### 5.1 Self-Consistent Loop Formulation
1. **Hamiltonian Construction**:
   $$\mathbf{H}_M^{(k)} = \mathbf{H}_0 + \operatorname{diag}\left( V_1^{(k)}, V_2^{(k)}, \dots, V_N^{(k)} \right)$$
2. **NEGF Retarded Green's Function**:
   $$\mathbf{G}^R(E) = \left[ (E + i\eta)\mathbf{I} - \mathbf{H}_M^{(k)} - \mathbf{\Sigma}_L(E) - \mathbf{\Sigma}_R(E) \right]^{-1}$$
3. **Non-Equilibrium Density Matrix Integration**:
   $$\rho_i = \frac{1}{2\pi} \int_{-\infty}^{\infty} \left[ \mathbf{A}_L(E) f_L(E) + \mathbf{A}_R(E) f_R(E) \right]_{ii} dE$$
4. **Poisson / Molecular Hubbard Charging Update**:
   $$V_i^{calc} = V_i^{gate} + \sum_j U_{ij} (\rho_j - \rho_j^0)$$
5. **Damped Linear / Anderson Mixing**:
   $$V_i^{(k+1)} = (1 - \alpha_{mix}) V_i^{(k)} + \alpha_{mix} V_i^{calc}, \quad \alpha_{mix} \in (0.05, 0.3]$$
   Iterated until maximum potential residual $\|\mathbf{V}^{(k+1)} - \mathbf{V}^{(k)}\|_\infty < \epsilon_{tol} \approx 10^{-5}\text{ eV}$.

---

## 6. Norm-Preserving Landau-Lifshitz-Gilbert-Slonczewski (LLGS) Solver

Micromagnetic spintronic simulation tracks the unit magnetization vector $\vec{m}(t)$ ($|\vec{m}| = 1$) under external, anisotropy, demagnetization, and dipole stray fields:
$$\frac{d\vec{m}}{dt} = -\frac{\gamma}{1+\alpha^2} \left[ \vec{m} \times \vec{B}_{eff} + \alpha\,\vec{m} \times (\vec{m} \times \vec{B}_{eff}) \right] + \vec{\tau}_{STT} + \vec{\tau}_{SOT}$$

### 6.1 Numerical RK4 Integration with Geometric Projection
Standard explicit Euler integration causes unphysical spiraling away from the unit sphere ($|\vec{m}| \ne 1$). Phonon employs 4th-order Runge-Kutta (RK4) integration with exact norm renormalization at every intermediate stage:
$$\vec{k}_1 = \mathbf{f}_{LLGS}(\vec{m}_n)$$
$$\vec{k}_2 = \mathbf{f}_{LLGS}\left( \frac{\vec{m}_n + \frac{\Delta t}{2}\vec{k}_1}{\left|\vec{m}_n + \frac{\Delta t}{2}\vec{k}_1\right|} \right)$$
$$\vec{k}_3 = \mathbf{f}_{LLGS}\left( \frac{\vec{m}_n + \frac{\Delta t}{2}\vec{k}_2}{\left|\vec{m}_n + \frac{\Delta t}{2}\vec{k}_2\right|} \right)$$
$$\vec{k}_4 = \mathbf{f}_{LLGS}\left( \frac{\vec{m}_n + \Delta t\,\vec{k}_3}{\left|\vec{m}_n + \Delta t\,\vec{k}_3\right|} \right)$$
$$\vec{m}_{n+1} = \frac{\vec{m}_n + \frac{\Delta t}{6}(\vec{k}_1 + 2\vec{k}_2 + 2\vec{k}_3 + \vec{k}_4)}{\left|\vec{m}_n + \frac{\Delta t}{6}(\vec{k}_1 + 2\vec{k}_2 + 2\vec{k}_3 + \vec{k}_4)\right|}$$
This guarantees strictly zero artificial damping or numerical magnetization drift ($||\vec{m}| - 1.0| < 10^{-15}$).

---

## 7. Cryogenic Quantum Error Correction (QEC) Decoding Engine

### 7.1 Real-Time Surface-Code Decoding ($d=3, d=5$)
Operating at $4\text{ K}$, `CryoQecDecoder` maps syndrome defect detection flags directly to corrective Pauli operators:
1. **Stabilizer Syndrome Ingestion**: Ingests boolean X-stabilizer bitmask (detecting Z errors) and Z-stabilizer bitmask (detecting X errors).
2. **Combinatorial Matching Logic**: Computes minimum-weight error chains via low-depth RSFQ combinatorial trees with propagation latency:
   $$\tau_{decode} = \lceil \log_2(N_{gates}) \rceil \cdot \tau_{stage} < 100\text{ ps} \quad (d=3)$$
3. **Pauli Operator Consolidation**: Combines detected $X$ and $Z$ errors on the same qubit: $X \cdot Z = Y$.
4. **Closed-Loop Feedback**: Drives cryogenic optoelectronic drivers sending optical pulse packets to qubit control lines, bypassing the $> 1\,\mu\text{s}$ classical room-temperature control bottleneck.

---

## 8. Multi-Threaded Rayon Parallelization Architectures

Phonon maximizes multi-core CPU throughput via zero-allocation Rayon patterns:
1. **Diakoptics & Circuit Tearing**: Decomposes large circuit graphs into $K$ weakly-coupled subcircuits solved concurrently across CPU worker threads, recombining boundary node voltages via the inter-subcircuit interconnect matrix.
2. **Parallel Neuromorphic Step**: Evaluates $N$ spiking optoelectronic neurons (`SoenNeuron`) in parallel via `par_iter_mut()`, checking somatic thresholding and optical pulse generation independently.
3. **Parallel QEC Syndrome Sweeps**: Evaluates tens of thousands of syndrome extraction rounds concurrently across threads with linear scaling.
4. **NSGA-II Transistor Population Evaluation**: Evaluates 100+ transistor candidate genomes (computing IV curves, subthreshold swings, and RF cutoffs) concurrently across all available hardware cores.

---

## 9. Topological Non-Abelian Braiding & Parity Conservation Solvers

### 9.1 Adiabatic T-Junction Exchange Protocol
To exchange two Majorana Zero Modes ($\gamma_1, \gamma_2$) without spatial collision, `MajoranaBraidingSolver` parameterizes a 3-stage electrostatic gate trajectory:
1. $\gamma_2$ is translated into the auxiliary junction stem by ramping $\mu_R: 0 \to 3\text{ meV}$ and $\mu_S: 3 \to 0\text{ meV}$.
2. $\gamma_1$ is translated through the central intersection from Left to Right by ramping $\mu_L: 0 \to 3\text{ meV}$ and $\mu_R: 3 \to 0\text{ meV}$.
3. $\gamma_2$ is retrieved from the Stem into the Left arm by ramping $\mu_S: 0 \to 3\text{ meV}$ and $\mu_L: 3 \to 0\text{ meV}$.

### 9.2 Adiabaticity & Landau-Zener Leakage
The solver continuously monitors the adiabatic parameter:
$$\eta_{ad}(t) = \frac{\hbar \, |\dot{\mu}(t)|}{\Delta_{top}^2}$$
Where $\Delta_{top} \approx 0.5\text{ meV}$ is the topological minigap protecting the Majoranas from exciting into the Bogoliubov quasiparticle continuum. The probability of non-adiabatic leakage is:
$$\mathcal{P}_{LZ} \approx \exp\left( -2\pi \frac{\Delta_{top}^2}{\hbar \, |\dot{\mu}|} \right) < 10^{-10}$$
Ensuring gate fidelities exceeding $99.9999\%$.

### 9.3 Stochastic Quasiparticle Poisoning (QPP) Tracker
Environmental stray thermal quasiparticles tunneling into the zero mode cause random parity transitions $P_{12} \to -P_{12}$. `FermionParitySolver` simulates this Poisson jump process with rate $\Gamma_{qp} \approx 1 - 10\text{ Hz}$ at $T \le 20\text{ mK}$, tracking topological lifetime $T_1^{topo} = 1/\Gamma_{qp}$ and dephasing $T_2^{topo} \approx 2 T_1^{topo}$.

