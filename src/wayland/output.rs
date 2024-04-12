use std::convert::TryInto;

use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};

use wayland_client::{globals::registry_queue_init, protocol::wl_output, Connection, QueueHandle};

pub fn monitor_size(conn: &Connection) -> Option<(u32, u32)> {
    // Now create an event queue and a handle to the queue so we can create objects.
    let (globals, mut event_queue) = registry_queue_init(conn).unwrap();
    let qh = event_queue.handle();

    // Initialize the registry handling so other parts of Smithay's client toolkit may bind
    // globals.
    let registry_state = RegistryState::new(&globals);

    // Initialize the delegate we will use for outputs.
    let output_delegate = OutputState::new(&globals, &qh);

    // Set up application state.
    //
    // This is where you will store your delegates and any data you wish to access/mutate while the
    // application is running.
    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
    };

    // `OutputState::new()` binds the output globals found in `registry_queue_init()`.
    //
    // After the globals are bound, we need to dispatch again so that events may be sent to the newly
    // created objects.
    event_queue.roundtrip(&mut list_outputs).unwrap();

    // Now our outputs have been initialized with data, we may access what outputs exist and information about
    // said outputs using the output delegate.
    list_outputs
        .output_state
        .outputs()
        .next()
        .and_then(|output| list_outputs.output_state.info(&output))
        .map(|info| {
            let (w, h) = info
                .modes
                .iter()
                .next()
                .map(|mode| mode.dimensions)
                .unwrap_or(info.physical_size);
            (w.try_into().unwrap(), h.try_into().unwrap())
        })
}

/// Application data.
///
/// This type is where the delegates for some parts of the protocol and any application specific data will
/// live.
struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
}

// In order to use OutputDelegate, we must implement this trait to indicate when something has happened to an
// output and to provide an instance of the output state to the delegate when dispatching events.
impl OutputHandler for ListOutputs {
    // First we need to provide a way to access the delegate.
    //
    // This is needed because delegate implementations for handling events use the application data type in
    // their function signatures. This allows the implementation to access an instance of the type.
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    // Then there exist these functions that indicate the lifecycle of an output.
    // These will be called as appropriate by the delegate implementation.

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

// Now we need to say we are delegating the responsibility of output related events for our application data
// type to the requisite delegate.
delegate_output!(ListOutputs);

// In order for our delegate to know of the existence of globals, we need to implement registry
// handling for the program. This trait will forward events to the RegistryHandler trait
// implementations.
delegate_registry!(ListOutputs);

// In order for delegate_registry to work, our application data type needs to provide a way for the
// implementation to access the registry state.
//
// We also need to indicate which delegates will get told about globals being created. We specify
// the types of the delegates inside the array.
impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        // Here we specify that OutputState needs to receive events regarding the creation and destruction of
        // globals.
        OutputState,
    }
}
