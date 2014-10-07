use network::ipv4::IPState;

/// Enables the given interface
pub fn _interface_up(state: IPState, interface_ix: uint){
    state.get_interface(interface_ix).enable();
}

/// Disables the given interface
pub fn _interface_down(state: IPState, interface_ix: uint){
    state.get_interface(interface_ix).disable();
}
