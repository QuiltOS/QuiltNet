use network::ipv4::state::IPState;

/// Enables the given interface
pub fn up(state: &IPState, interface_ix: uint){
    println!("control:: up {}", interface_ix);
    match state.get_interface(interface_ix) {
        Some(&(_,_, ref interface)) => interface.enable(),
        None => ()
    }
}

/// Disables the given interface
pub fn down(state: &IPState, interface_ix: uint){
    println!("control:: down {}", interface_ix);
    match state.get_interface(interface_ix) {
        Some(&(_, _, ref interface)) => interface.disable(),
        None => ()
    }
}
