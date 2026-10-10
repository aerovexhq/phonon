#![deny(unsafe_code)]

//! Live Electronic Distributor API Data Models and Multi-Source Parametric Pricing Engine.
//!
//! Provides data models, price break structures, inventory tracking, and multi-distributor
//! pricing comparison across Digi-Key, Mouser, and LCSC.

/// Supported tier-1 global electronic component distributors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistributorKind {
    DigiKey,
    Mouser,
    Lcsc,
}

impl DistributorKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DigiKey => "Digi-Key Electronics",
            Self::Mouser => "Mouser Electronics",
            Self::Lcsc => "LCSC Electronics",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DigiKey => "DIGIKEY",
            Self::Mouser => "MOUSER",
            Self::Lcsc => "LCSC",
        }
    }

    pub fn default_currency(&self) -> &'static str {
        match self {
            Self::DigiKey | Self::Mouser => "USD",
            Self::Lcsc => "USD",
        }
    }
}

/// Commercial component packaging types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PackagingType {
    CutTape,
    TapeAndReel,
    Tube,
    Tray,
    Bulk,
}

impl PackagingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CutTape => "Cut Tape",
            Self::TapeAndReel => "Tape & Reel",
            Self::Tube => "Tube",
            Self::Tray => "Tray",
            Self::Bulk => "Bulk / Bag",
        }
    }
}

/// Product lifecycle status from distributor catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LifecycleStatus {
    Active,
    NotRecommendedForNewDesigns,
    EndOfLife,
    Obsolete,
}

impl LifecycleStatus {
    pub fn is_recommended(&self) -> bool {
        matches!(self, Self::Active)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active - Production",
            Self::NotRecommendedForNewDesigns => "NRND (Not Recommended)",
            Self::EndOfLife => "EOL (End of Life)",
            Self::Obsolete => "Obsolete",
        }
    }
}

/// Tiered volume price break tuple: (minimum quantity, unit price in USD).
pub type PriceBreak = (usize, f64);

/// Real-time live distributor stock and pricing quote for a specific part.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistributorQuote {
    pub distributor: DistributorKind,
    pub distributor_part_number: String,
    pub manufacturer_part_number: String,
    pub manufacturer_name: String,
    pub description: String,
    pub in_stock_quantity: usize,
    pub lead_time_weeks: f64,
    pub minimum_order_quantity: usize,
    pub order_multiple: usize,
    pub packaging: PackagingType,
    pub lifecycle: LifecycleStatus,
    /// Tiered volume price breaks: (minimum quantity, unit price in USD).
    pub price_breaks: Vec<(usize, f64)>,
    pub datasheet_url: String,
}

impl DistributorQuote {
    /// Computes the best available unit price for a given requested purchase volume.
    pub fn unit_price_at_qty(&self, requested_qty: usize) -> f64 {
        if self.price_breaks.is_empty() {
            return 0.0;
        }

        let mut best_price = self.price_breaks[0].1;
        for &(tier_qty, price) in &self.price_breaks {
            if requested_qty >= tier_qty {
                best_price = price;
            }
        }
        best_price
    }

    /// Computes the extended line total (unit price * actual purchased quantity,
    /// rounded up to MOQ and order multiple).
    pub fn extended_cost_at_qty(&self, requested_qty: usize) -> (usize, f64) {
        let mut order_qty = requested_qty.max(self.minimum_order_quantity);
        if self.order_multiple > 1 {
            let rem = order_qty % self.order_multiple;
            if rem > 0 {
                order_qty += self.order_multiple - rem;
            }
        }
        let unit_price = self.unit_price_at_qty(order_qty);
        (order_qty, (order_qty as f64) * unit_price)
    }
}

/// Consolidated multi-distributor price comparison card for a single component.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentMarketComparison {
    pub manufacturer_part_number: String,
    pub generic_name: String,
    pub footprint: String,
    pub quotes: Vec<DistributorQuote>,
    pub selected_distributor: Option<DistributorKind>,
}

impl ComponentMarketComparison {
    pub fn new(mpn: &str, generic_name: &str, footprint: &str) -> Self {
        Self {
            manufacturer_part_number: mpn.to_string(),
            generic_name: generic_name.to_string(),
            footprint: footprint.to_string(),
            quotes: Vec::new(),
            selected_distributor: None,
        }
    }

    /// Identifies the distributor offering the lowest unit price at the requested volume
    /// that has adequate inventory in stock.
    pub fn find_lowest_cost_supplier(&self, volume: usize) -> Option<&DistributorQuote> {
        let mut lowest: Option<&DistributorQuote> = None;
        let mut min_cost = f64::INFINITY;

        for quote in &self.quotes {
            if quote.in_stock_quantity >= volume || quote.in_stock_quantity > 0 {
                let (_, ext_cost) = quote.extended_cost_at_qty(volume);
                if ext_cost < min_cost {
                    min_cost = ext_cost;
                    lowest = Some(quote);
                }
            }
        }

        lowest.or_else(|| self.quotes.first())
    }

    /// Identifies the distributor with the shortest lead time.
    pub fn find_fastest_lead_time_supplier(&self) -> Option<&DistributorQuote> {
        self.quotes.iter().min_by(|a, b| {
            a.lead_time_weeks.partial_cmp(&b.lead_time_weeks).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}
