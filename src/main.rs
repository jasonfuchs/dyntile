use river_layout::prelude::*;

struct Dyntile;

impl LayoutGenerator for Dyntile {
    const NAMESPACE: &'static str = "dyntile";
}

fn main() -> Result<(), Error> {
    Dyntile.run()
}
