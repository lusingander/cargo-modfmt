mod private;

pub(crate) mod crate_visible;
pub mod zebra;

mod outer {
    mod private;

    pub(in crate::outer) mod ancestor_visible;
    pub(super) mod parent_visible;
    pub(self) mod self_visible;

    /// Child module documentation.
    pub mod child {
        mod private;

        pub(in crate::outer) mod ancestor_visible;

        pub fn value() -> u32 {
            private::VALUE
        }
    }

    pub fn value() -> u32 {
        ancestor_visible::VALUE
            + self_visible::VALUE
            + private::VALUE
            + child::value()
            + child::ancestor_visible::VALUE
    }
}

#[cfg(test)]
mod tests {
    mod a;
    mod z;

    pub fn value() -> u32 {
        z::VALUE + a::VALUE
    }
}

#[doc = "Module documentation."]
mod documented {
    mod a;
    mod z;

    pub fn value() -> u32 {
        z::VALUE + a::VALUE
    }
}

#[cfg(any())]
mod excluded {
    mod absent;
    mod missing;
}

fn main() {
    let value = zebra::VALUE
        + private::VALUE
        + crate_visible::VALUE
        + outer::parent_visible::VALUE
        + outer::value()
        + documented::value();
    #[cfg(test)]
    let value = value + tests::value();
    println!("{value}");
}
