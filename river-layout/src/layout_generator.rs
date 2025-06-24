use crate::prelude::*;

use crate::protocol::river_layout_manager_v3;
use crate::protocol::river_layout_v3;

use wayland_client::Connection;
use wayland_client::Dispatch;
use wayland_client::QueueHandle;
use wayland_client::delegate_noop;

use wayland_client::protocol::wl_output;
use wayland_client::protocol::wl_registry;

pub trait LayoutGenerator: 'static {
    const NAMESPACE: &'static str;
    fn run(self) -> Result<(), Error>
    where
        Self: Sized,
    {
        let conn = Connection::connect_to_env()?;

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();
        display.get_registry(&qh, ());

        let mut state = State::new(self);

        while !state.should_exit() {
            event_queue.blocking_dispatch(&mut state)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum State<LG: LayoutGenerator> {
    Uninit(LG),
    Init {
        layout_manager: river_layout_manager_v3::RiverLayoutManagerV3,
        layout_generator: LG,
        tags: Option<u32>,
    },
    ShouldExit,
}

impl<LG: LayoutGenerator> State<LG> {
    fn new(layout_generator: LG) -> Self {
        Self::Uninit(layout_generator)
    }

    fn should_exit(&self) -> bool {
        match self {
            Self::ShouldExit => true,
            _ => false,
        }
    }
}

impl<LG: LayoutGenerator> Dispatch<wl_registry::WlRegistry, ()> for State<LG> {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);

        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match (&state, &interface[..]) {
                (_, "wl_output") => {
                    let _: wl_output::WlOutput = registry.bind(name, version, &qh, ());
                }
                (Self::Uninit(layout_generator), "river_layout_manager_v3") => {
                    let layout_manager: river_layout_manager_v3::RiverLayoutManagerV3 =
                        registry.bind(name, version, &qh, ());

                    unsafe {
                        use std::mem;
                        use std::ptr;

                        let old = ptr::read(layout_generator);

                        // don't accidentally drop the layout generator
                        mem::forget(mem::replace(
                            state,
                            Self::Init {
                                layout_manager,
                                layout_generator: old,
                                tags: None,
                            },
                        ));
                    }
                }
                (_, _) => (),
            }
        }
    }
}

impl<LG: LayoutGenerator> Dispatch<wl_output::WlOutput, ()> for State<LG> {
    fn event(
        state: &mut Self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        dbg!(&event);

        match (state, event) {
            (Self::Init { layout_manager, .. }, wl_output::Event::Name { name }) => {
                let _layout: river_layout_v3::RiverLayoutV3 =
                    layout_manager.get_layout(output, LG::NAMESPACE.into(), &qh, name);
            }
            (_, _) => (),
        }
    }
}

delegate_noop!(@<LG: LayoutGenerator> State<LG>: ignore river_layout_manager_v3::RiverLayoutManagerV3 );

impl<LG: LayoutGenerator> Dispatch<river_layout_v3::RiverLayoutV3, String> for State<LG> {
    fn event(
        state: &mut Self,
        layout: &river_layout_v3::RiverLayoutV3,
        event: river_layout_v3::Event,
        output_name: &String,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let Self::Init {
            layout_generator,
            tags,
            ..
        } = state
        {
            match event {
                river_layout_v3::Event::NamespaceInUse => *state = Self::ShouldExit,
                river_layout_v3::Event::UserCommandTags {
                    tags: user_command_tags,
                } => *tags = Some(user_command_tags),
                river_layout_v3::Event::UserCommand { command } => (),
                river_layout_v3::Event::LayoutDemand {
                    view_count,
                    usable_width,
                    usable_height,
                    tags,
                    serial,
                } => (),
            }
        }
    }
}
