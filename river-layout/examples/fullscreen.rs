use river_layout::GeneratedLayout;
use river_layout::LayoutGenerator;
use river_layout::ViewDimensions;

fn main() -> Result<(), river_layout::Error<Fullscreen>> {
    Fullscreen.run()
}

struct Fullscreen;

impl LayoutGenerator for Fullscreen {
    const NAMESPACE: &'static str = "fullscreen";
    type Err = std::io::Error;

    fn cmd(&mut self, _tags: Option<u32>, _output: &str, _cmd: &str) -> Result<(), Self::Err> {
        Ok(())
    }

    fn generate_layout(
        &mut self,
        _tags: u32,
        _output: &str,
        usable_space: (u32, u32),
        view_count: usize,
    ) -> Result<GeneratedLayout, Self::Err> {
        let views = std::iter::repeat(ViewDimensions {
            location: (0, 0),
            space: usable_space,
        });

        Ok(GeneratedLayout {
            views: views.take(view_count).collect(),
            name: "[ ]".into(),
        })
    }
}
