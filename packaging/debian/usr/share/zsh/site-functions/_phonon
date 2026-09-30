#compdef phonon

_phonon() {
    local -a commands
    commands=(
        'ui:Launch native GPU-accelerated desktop CAD visual studio'
        'gui:Launch native GPU-accelerated desktop CAD visual studio'
        'validate:Validate circuit topology and electrical rules (ERC)'
        'run:Execute simulation (.OP, .DC, .TRAN) and stream telemetry'
        'sweep:Execute parallel parametric component sweep'
        'mc:Execute Monte Carlo statistical tolerance analysis'
        'help:Print help message or subcommand help'
    )

    _arguments -C \
        '(-h --help)'{-h,--help}'[Print help]' \
        '(-V --version)'{-V,--version}'[Print version]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'phonon commands' commands
            ;;
        args)
            case $line[1] in
                validate)
                    _arguments \
                        '1:SPICE netlist file:_files'
                    ;;
                run)
                    _arguments \
                        '(-f --format)'{-f,--format}'[Telemetry format]:format:(csv json binary)' \
                        '(-o --output)'{-o,--output}'[Output file destination]:output file:_files' \
                        '1:SPICE netlist file:_files'
                    ;;
                sweep)
                    _arguments \
                        '--param[Parameter name to sweep]:parameter name:' \
                        '--start[Starting value]:start value:' \
                        '--stop[Stopping value]:stop value:' \
                        '--steps[Sweep steps]:steps:' \
                        '(-f --format)'{-f,--format}'[Telemetry format]:format:(csv json binary)' \
                        '(-o --output)'{-o,--output}'[Output destination]:output file:_files' \
                        '1:SPICE netlist file:_files'
                    ;;
                mc)
                    _arguments \
                        '(-s --samples)'{-s,--samples}'[Number of Monte Carlo samples]:samples:' \
                        '(-t --tol)'{-t,--tol}'[Component tolerance fraction]:tolerance:' \
                        '--seed[RNG seed]:seed:' \
                        '(-f --format)'{-f,--format}'[Telemetry format]:format:(csv json binary)' \
                        '(-o --output)'{-o,--output}'[Output destination]:output file:_files' \
                        '1:SPICE netlist file:_files'
                    ;;
            esac
            ;;
    esac
}

_phonon "$@"
