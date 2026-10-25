pub mod zebra;
mod private;
pub(crate) mod crate_visible;

mod outer {
    pub(super) mod parent_visible;
    pub(in crate::outer) mod ancestor_visible;
    pub(self) mod self_visible;
    mod private;

    /// Child module documentation.
    pub mod child {
        pub(in crate::outer) mod ancestor_visible;
        mod private;

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
    mod z;
    mod a;

    pub fn value() -> u32 {
        z::VALUE + a::VALUE
    }
}

#[doc = "Module documentation."]
mod documented {
    mod z;
    mod a;

    pub fn value() -> u32 {
        z::VALUE + a::VALUE
    }
}

#[cfg(any())]
mod excluded {
    mod missing;
    mod absent;
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
