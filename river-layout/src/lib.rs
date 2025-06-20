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

use wayland_client::ConnectError;
use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::globals::BindError;
use wayland_client::globals::GlobalError;
use wayland_client::globals::GlobalListContents;
use wayland_client::globals::registry_queue_init;

use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_registry;

use protocol::river_layout_manager_v3;
use protocol::river_layout_v3;

#[derive(Debug)]
pub struct GeneratedLayout {
    views: Vec<ViewDimensions>,
    name: String,
}

#[derive(Debug)]
pub struct ViewDimensions {
    location: (i32, i32),
    space: (u32, u32),
}

pub trait LayoutGenerator: Sized + 'static {
    const NAMESPACE: &'static str;
    type Err: std::error::Error;

    fn cmd(&mut self, tags: Option<u32>, output: &str, cmd: &str) -> Result<(), Self::Err>;
    fn generate_layout(
        &mut self,
        tags: u32,
        output: &str,
        usable_space: (u32, u32),
        view_count: usize,
    ) -> Result<GeneratedLayout, Self::Err>;

    fn run(self) -> Result<(), Error<Self>> {
        let conn = Connection::connect_to_env()?;
        let (globals, queue) = registry_queue_init::<State<Self>>(&conn)?;
        let river_layout_manager_v3 = globals.bind(&queue.handle(), 1..=2, ())?;

        let state = State {
            river_layout_manager_v3,
            layout_generator: self,
            tags: None,
            error: None,
        };

        'event_loop: loop {
            if let Some(ref err) = state.error {
                match err {
                    _ => break 'event_loop,
                }
            }
        }

        state.error.map_or(Ok(()), Err)
    }
}

pub enum Error<LG: LayoutGenerator> {
    Connect(ConnectError),
    Global(GlobalError),
    Bind(BindError),
    NamespaceInUse(&'static str),
    LayoutGenerator(LG::Err),
}

impl<LG: LayoutGenerator> std::fmt::Debug for Error<LG> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::Connect(err) => fmt.debug_tuple("Connect").field(err).finish(),
            Error::Global(err) => fmt.debug_tuple("Global").field(err).finish(),
            Error::Bind(err) => fmt.debug_tuple("Bind").field(err).finish(),
            Error::NamespaceInUse(ns) => fmt.debug_tuple("NamespaceInUse").field(ns).finish(),
            Error::LayoutGenerator(err) => fmt.debug_tuple("LayoutGenerator").field(err).finish(),
        }
    }
}

impl<LG: LayoutGenerator> std::fmt::Display for Error<LG> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::Connect(err) => write!(f, "{err}"),
            Error::Global(err) => write!(f, "{err}"),
            Error::Bind(err) => write!(f, "{err}"),
            Error::NamespaceInUse(ns) => {
                write!(f, "the requested namespace \"{ns}\" is already in use")
            }
            Error::LayoutGenerator(err) => write!(f, "Layout generator error: {err}"),
        }
    }
}

impl<LG: LayoutGenerator> std::error::Error for Error<LG> {}

impl<LG: LayoutGenerator> From<ConnectError> for Error<LG> {
    fn from(err: ConnectError) -> Self {
        Error::Connect(err)
    }
}

impl<LG: LayoutGenerator> From<GlobalError> for Error<LG> {
    fn from(err: GlobalError) -> Self {
        Error::Global(err)
    }
}

impl<LG: LayoutGenerator> From<BindError> for Error<LG> {
    fn from(err: BindError) -> Self {
        Error::Bind(err)
    }
}

#[derive(Debug)]
pub struct State<LG: LayoutGenerator> {
    river_layout_manager_v3: river_layout_manager_v3::RiverLayoutManagerV3,
    layout_generator: LG,
    tags: Option<u32>,
    error: Option<Error<LG>>,
}

impl<LG: LayoutGenerator> Dispatch<wl_registry::WlRegistry, GlobalListContents> for State<LG> {
    fn event(
        _state: &mut State<LG>,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _udata: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<State<LG>>,
    ) {
        use wl_registry::Event;

        match event {
            Event::Global {
                name,
                interface,
                version,
            } if interface == "wl_output" => {
                let _wl_output: wl_output::WlOutput = proxy.bind(name, version, qh, OutputData {});
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
struct OutputData {}

impl<LG: LayoutGenerator> Dispatch<wl_output::WlOutput, OutputData> for State<LG> {
    fn event(
        state: &mut State<LG>,
        proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _udata: &OutputData,
        _conn: &Connection,
        qh: &QueueHandle<State<LG>>,
    ) {
        use wl_output::Event;

        match event {
            Event::Name { name } => {
                let _river_layout_v3 = state.river_layout_manager_v3.get_layout(
                    &proxy,
                    LG::NAMESPACE.into(),
                    qh,
                    LayoutData { output_name: name },
                );
            }
            _ => (),
        }
    }
}

#[derive(Debug)]
struct LayoutData {
    output_name: String,
}

impl<LG: LayoutGenerator> Dispatch<river_layout_manager_v3::RiverLayoutManagerV3, ()>
    for State<LG>
{
    fn event(
        _state: &mut State<LG>,
        _proxy: &river_layout_manager_v3::RiverLayoutManagerV3,
        _event: river_layout_manager_v3::Event,
        _udata: &(),
        _conn: &Connection,
        _qh: &QueueHandle<State<LG>>,
    ) {
    }
}

impl<LG: LayoutGenerator> Dispatch<river_layout_v3::RiverLayoutV3, LayoutData> for State<LG> {
    fn event(
        state: &mut State<LG>,
        proxy: &river_layout_v3::RiverLayoutV3,
        event: river_layout_v3::Event,
        udata: &LayoutData,
        _conn: &Connection,
        _qh: &QueueHandle<State<LG>>,
    ) {
        use river_layout_v3::Event;

        match event {
            Event::NamespaceInUse => state.error = Some(Error::NamespaceInUse(LG::NAMESPACE)),
            Event::UserCommandTags { tags } => state.tags = Some(tags),
            Event::UserCommand { command } => {
                if let Err(err) =
                    state
                        .layout_generator
                        .cmd(state.tags, &udata.output_name, &command)
                {
                    eprintln!("Error: {err}");
                }
            }
            Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags,
                serial,
            } => {
                fn layout_demand<LG: LayoutGenerator>(
                    state: &mut State<LG>,
                    proxy: &river_layout_v3::RiverLayoutV3,
                    udata: &LayoutData,

                    view_count: u32,
                    usable_width: u32,
                    usable_height: u32,
                    tags: u32,
                    serial: u32,
                ) -> Result<(), Error<LG>> {
                    let view_count = view_count as usize;
                    let generated_layout = state
                        .layout_generator
                        .generate_layout(
                            tags,
                            &udata.output_name,
                            (usable_width, usable_height),
                            view_count,
                        )
                        .map_err(Error::LayoutGenerator)?;

                    assert_eq!(generated_layout.views.len(), view_count);

                    for view in generated_layout.views {
                        let ViewDimensions {
                            location: (x, y),
                            space: (width, height),
                        } = view;

                        proxy.push_view_dimensions(x, y, width, height, serial);
                    }

                    proxy.commit(generated_layout.name, serial);

                    Ok(())
                }

                state.error = layout_demand(
                    state,
                    proxy,
                    udata,
                    view_count,
                    usable_width,
                    usable_height,
                    tags,
                    serial,
                )
                .err();
            }
        }
    }
}
