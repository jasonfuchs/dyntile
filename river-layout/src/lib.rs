pub mod protocol {
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("protocol/river-layout-v3.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("protocol/river-layout-v3.xml");
}

use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::globals::GlobalListContents;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_registry;
use wayland_client::protocol::wl_registry::WlRegistry;

use protocol::river_layout_manager_v3;
use protocol::river_layout_manager_v3::RiverLayoutManagerV3;
use protocol::river_layout_v3;
use protocol::river_layout_v3::RiverLayoutV3;

#[derive(Debug)]
pub struct View {
    location: (i32, i32),
    space: (u32, u32),
}

#[derive(Debug)]
pub struct GeneratedLayout<const N: usize> {
    name: Box<str>,
    views: [View; N],
}

#[derive(Debug)]
struct State<L: LayoutGenerator> {
    layout_manager: RiverLayoutManagerV3,
    layout_generator: L,
    tags: Option<u32>,
    outputs: Vec<Output>,
    exit: bool,
}

#[derive(Debug)]
struct Output {
    wl_output: WlOutput,
    name: u32,
    layout: Option<Layout>,
}

#[derive(Debug)]
struct Layout {
    river_layout: RiverLayoutV3,
    output_name: String,
}

pub trait LayoutGenerator: Sized + 'static {
    const NAMESPACE: &'static str;
    type Err: std::error::Error;

    fn user_cmd(&mut self, tags: Option<u32>, output: &str, cmd: &str) -> Result<(), Self::Err>;
    fn generate_layout<const N: usize>(
        &mut self,
        tags: u32,
        output: &str,
        usable_space: (u32, u32),
    ) -> Result<GeneratedLayout<N>, Self::Err>;

    fn run(self) -> Result<(), Self::Err> {
        let conn = Connection::connect_to_env().unwrap(); // FIXME error handling
        let (globals, queue) = registry_queue_init::<State<Self>>(&conn).unwrap();

        let layout_manager = globals.bind(&queue.handle(), 1..=2, ()).unwrap();

        let state = State {
            layout_manager,
            layout_generator: self,
            tags: None,
            outputs: vec![],
            exit: false,
        };

        todo!()
    }
}

impl<L: LayoutGenerator> Dispatch<WlRegistry, GlobalListContents> for State<L> {
    fn event(
        state: &mut Self,
        wl_registry: &WlRegistry,
        event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        queue: &QueueHandle<State<L>>,
    ) {
        use wl_registry::Event;

        match event {
            Event::Global {
                name,
                interface,
                version,
            } if interface == "wl_output" => {
                let wl_output: WlOutput = wl_registry.bind(name, version.min(4), queue, ());
                state.outputs.push(Output {
                    wl_output,
                    name,
                    layout: None,
                });
            }
            Event::GlobalRemove { name } => {
                if let Some(i) = state.outputs.iter().position(|output| output.name == name) {
                    let output = state.outputs.remove(i);

                    // TODO add to drop implementation
                    if let Some(layout) = output.layout {
                        layout.river_layout.destroy();
                    }

                    output.wl_output.release();
                }
            }
            _ => (),
        }
    }
}

impl<L: LayoutGenerator> Dispatch<WlOutput, ()> for State<L> {
    fn event(
        state: &mut Self,
        wl_output: &WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        queue: &QueueHandle<State<L>>,
    ) {
        let output = state
            .outputs
            .iter_mut()
            .find(|output| &output.wl_output == wl_output)
            .unwrap();

        use wl_output::Event;

        if let Event::Name { name: output_name } = event {
            let river_layout =
                state
                    .layout_manager
                    .get_layout(&wl_output, L::NAMESPACE.into(), queue, ());

            let layout = Layout {
                river_layout,
                output_name,
            };

            output.layout = Some(layout);
        }
    }
}

impl<L: LayoutGenerator> Dispatch<RiverLayoutManagerV3, ()> for State<L> {
    fn event(
        _state: &mut Self,
        _proxy: &RiverLayoutManagerV3,
        _event: river_layout_manager_v3::Event,
        _data: &(),
        _conn: &Connection,
        _queue: &QueueHandle<State<L>>,
    ) {
    }
}

impl<L: LayoutGenerator> Dispatch<RiverLayoutV3, ()> for State<L> {
    fn event(
        state: &mut Self,
        river_layout: &RiverLayoutV3,
        event: river_layout_v3::Event,
        _data: &(),
        _conn: &Connection,
        _queue: &QueueHandle<State<L>>,
    ) {
        let layout = state
            .outputs
            .iter()
            .filter_map(|output| output.layout.as_ref())
            .find(|layout| &layout.river_layout == river_layout)
            .unwrap();

        use river_layout_v3::Event;

        match event {
            Event::NamespaceInUse => state.exit = true,
            Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags,
                serial,
            } => {}
            Event::UserCommand { command } => {
                state
                    .layout_generator
                    .user_cmd(state.tags, &layout.output_name, &command)
                    .unwrap();
            }
            Event::UserCommandTags { tags } => state.tags = Some(tags),
        }
    }
}
