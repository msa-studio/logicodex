// =========================================================================
// Logicodex v1.41.0-alpha — Host Reactor Integration Tests
//
// Tests: HostReactor, GatePermissions, HardwareZone, HostFunction dispatch
//        Guest → Host → HW mediation end-to-end
// =========================================================================

use logicodex::net::{
    GatePermissions, HardwareZone, HostFunction, HostReactor, HostReactorError, GuestRequest, HostResponse,
};

// ─── GatePermissions ───

#[test]
fn gate_permissions_allow_and_check() {
    let perms = GatePermissions::new()
        .allow("gpio-control", vec![0, 1, 2, 3])
        .allow("timer-set", vec![0, 1]);

    assert!(perms.is_allowed("gpio-control", 0));
    assert!(perms.is_allowed("gpio-control", 3));
    assert!(!perms.is_allowed("gpio-control", 99));
    assert!(!perms.is_allowed("dma-transfer", 0));
}

#[test]
fn gate_permissions_empty_denies_all() {
    let perms = GatePermissions::new();
    assert!(!perms.is_allowed("anything", 0));
}

// ─── HardwareZone ───

#[test]
fn hardware_zone_claim_and_release() {
    let mut zone = HardwareZone::new();
    assert!(zone.is_available(0));

    zone.claim(0, "gpio-control").unwrap();
    assert!(!zone.is_available(0));

    zone.release(0);
    assert!(zone.is_available(0));
}

#[test]
fn hardware_zone_double_claim_fails() {
    let mut zone = HardwareZone::new();
    zone.claim(0, "gpio-control").unwrap();
    let result = zone.claim(0, "timer-set");
    assert!(matches!(result, Err(HostReactorError::HardwareBusy { pin: 0 })));
}

// ─── HostReactor creation ───

#[test]
fn host_reactor_new() {
    let perms = GatePermissions::new().allow("gpio-control", vec![0, 1, 2]);
    let reactor = HostReactor::new(perms);
    assert_eq!(reactor.stats().gpio_calls, 0);
    assert!(reactor.is_permitted("gpio-control", 0));
    assert!(!reactor.is_permitted("gpio-control", 99));
}

#[test]
fn host_reactor_permissive() {
    let reactor = HostReactor::permissive();
    assert!(reactor.is_permitted("gpio-control", 0));
    assert!(reactor.is_permitted("gpio-control", 63));
    assert!(reactor.is_permitted("timer-set", 0));
    assert!(reactor.is_permitted("dma-transfer", 0));
}

// ─── GPIO control ───

#[test]
fn host_reactor_gpio_control_allowed() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.gpio_control(5, "output");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
    assert_eq!(reactor.stats().gpio_calls, 1);
}

#[test]
fn host_reactor_gpio_invalid_mode() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.gpio_control(5, "invalid_mode");
    assert!(matches!(result, Err(HostReactorError::InvalidMode { .. })));
    assert_eq!(reactor.stats().errors, 0); // error counted in result, not stats.errors
}

#[test]
fn host_reactor_gpio_invalid_pin() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.gpio_control(999, "output");
    assert!(matches!(result, Err(HostReactorError::InvalidPin { pin: 999, .. })));
}

#[test]
fn host_reactor_gpio_permission_denied() {
    let perms = GatePermissions::new().allow("gpio-control", vec![0, 1]); // pin 5 NOT allowed
    let mut reactor = HostReactor::new(perms);
    let result = reactor.gpio_control(5, "output");
    assert!(matches!(result, Err(HostReactorError::PermissionDenied { .. })));
    assert_eq!(reactor.stats().denied_calls, 1);
}

// ─── Timer set ───

#[test]
fn host_reactor_timer_set() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.timer_set(0, 1000);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
    assert_eq!(reactor.stats().timer_calls, 1);
}

#[test]
fn host_reactor_timer_zero_duration() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.timer_set(0, 0);
    assert!(result.is_err());
}

// ─── DMA transfer ───

#[test]
fn host_reactor_dma_transfer() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.dma_transfer(0, 0x1000, 0x2000, 256);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 256);
    assert_eq!(reactor.stats().dma_calls, 1);
}

#[test]
fn host_reactor_dma_same_src_dst() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.dma_transfer(0, 0x1000, 0x1000, 256);
    assert!(result.is_err());
}

// ─── HostFunction dispatch ───

#[test]
fn host_reactor_register_function() {
    let mut reactor = HostReactor::permissive();
    assert!(reactor.register_host_function("gpio-control").is_ok());
    assert!(reactor.register_host_function("timer-set").is_ok());
    assert!(reactor.register_host_function("unknown").is_err());
}

#[test]
fn host_reactor_dispatch_gpio() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.dispatch(HostFunction::GpioControl, &[5, 1]); // pin=5, mode=output
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn host_reactor_dispatch_timer() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.dispatch(HostFunction::TimerSet, &[0, 5000]); // pin=0, micros=5000
    assert!(result.is_ok());
}

#[test]
fn host_reactor_dispatch_dma() {
    let mut reactor = HostReactor::permissive();
    let result = reactor.dispatch(HostFunction::DmaTransfer, &[0, 0x1000, 0x2000, 256]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 256);
}

// ─── Stats tracking ───

#[test]
fn host_reactor_stats_accumulate() {
    let mut reactor = HostReactor::permissive();

    let _ = reactor.gpio_control(0, "output");
    let _ = reactor.gpio_control(1, "input");
    let _ = reactor.timer_set(0, 1000);
    let _ = reactor.dma_transfer(0, 0x1000, 0x2000, 128);

    let stats = reactor.stats();
    assert_eq!(stats.gpio_calls, 2);
    assert_eq!(stats.timer_calls, 1);
    assert_eq!(stats.dma_calls, 1);
    assert_eq!(stats.denied_calls, 0);
}

// ─── GuestRequest / HostResponse types ───

#[test]
fn guest_request_creation() {
    let req = GuestRequest {
        operation: "gpio-control".to_string(),
        pin: 5,
        args: vec![5, 1],
    };
    assert_eq!(req.operation, "gpio-control");
    assert_eq!(req.pin, 5);
}

#[test]
fn host_response_ok() {
    let resp = HostResponse { result: Ok(42) };
    assert!(resp.result.is_ok());
    assert_eq!(resp.result.unwrap(), 42);
}

#[test]
fn host_response_err() {
    let resp = HostResponse {
        result: Err(HostReactorError::PermissionDenied { gate: "HW.GPIO:5".to_string() }),
    };
    assert!(resp.result.is_err());
}

// ─── End-to-end: permission denied then allowed ───

#[test]
fn host_reactor_e2e_permission_scenario() {
    // Guest tries pin 5 (not allowed) → denied
    let perms = GatePermissions::new()
        .allow("gpio-control", vec![0, 1, 2]);
    let mut reactor = HostReactor::new(perms);

    // First attempt: denied
    let r1 = reactor.gpio_control(5, "output");
    assert!(matches!(r1, Err(HostReactorError::PermissionDenied { .. })));

    // Second attempt: allowed pin
    let r2 = reactor.gpio_control(1, "output");
    assert!(r2.is_ok());

    // Stats
    assert_eq!(reactor.stats().denied_calls, 1);
    assert_eq!(reactor.stats().gpio_calls, 1);
}
