use core::cell::RefCell;
use lib::safe::Safe;
use x86_64::structures::idt::*;

type IDT = Safe<RefCell<InterruptDescriptorTable>>;

static IDT: IDT = Safe::new(RefCell::new(InterruptDescriptorTable::new()));

/// Initializes the Interrupt Descriptor Table (IDT).
pub fn init() {
    let mut idt = IDT.borrow_mut();
    idt.divide_error.set_handler_fn(exception_handlers::divide);
    idt.debug.set_handler_fn(exception_handlers::debug);
    idt.non_maskable_interrupt
        .set_handler_fn(exception_handlers::nmi);
    idt.breakpoint
        .set_handler_fn(exception_handlers::breakpoint);
    idt.overflow.set_handler_fn(exception_handlers::overflow);
    idt.bound_range_exceeded
        .set_handler_fn(exception_handlers::bound_range_exceeded);
    idt.invalid_opcode
        .set_handler_fn(exception_handlers::invalid_opcode);
    idt.device_not_available
        .set_handler_fn(exception_handlers::device_not_available);
    idt.double_fault
        .set_handler_fn(exception_handlers::double_fault);
    idt.invalid_tss
        .set_handler_fn(exception_handlers::invalid_tss);
    idt.segment_not_present
        .set_handler_fn(exception_handlers::segment_not_present);
    idt.stack_segment_fault
        .set_handler_fn(exception_handlers::stack_segment_fault);
    idt.general_protection_fault
        .set_handler_fn(exception_handlers::general_protection_fault);
    idt.page_fault
        .set_handler_fn(exception_handlers::page_fault);
    idt.x87_floating_point
        .set_handler_fn(exception_handlers::x87_floating_point);
    idt.alignment_check
        .set_handler_fn(exception_handlers::alignment_check);
    idt.machine_check
        .set_handler_fn(exception_handlers::machine_check);
    idt.simd_floating_point
        .set_handler_fn(exception_handlers::simd_floating_point);
    idt.virtualization
        .set_handler_fn(exception_handlers::virtualization);
    idt.cp_protection_exception
        .set_handler_fn(exception_handlers::cp_protection_exception);
    idt.hv_injection_exception
        .set_handler_fn(exception_handlers::hv_injection_exception);
    idt.vmm_communication_exception
        .set_handler_fn(exception_handlers::vmm_communication_exception);
    idt.security_exception
        .set_handler_fn(exception_handlers::security_exception);

    /* TODO add interrupt handlers.
     for i in 0..256 /* vectors */ - 32 /* exceptions */ {
        idt[i] = interrupt_handler;
    }*/

    unsafe {
        idt.load_unsafe();
    }
}

mod exception_handlers {
    use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

    pub extern "x86-interrupt" fn divide(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x0 division error, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn debug(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x1 debug, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn nmi(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x2 nmi, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x3 breakpoint, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x4 overflow, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x5 bound_range_exceeded, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x6 invalid_opcode, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn device_not_available(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x7 device_not_available, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn double_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) -> ! {
        panic!(
            "exception: 0x8 double_fault, error_code={error_code:?}, stack_frame={stack_frame:#?}"
        );

        // TODO triple fault detection
    }

    pub extern "x86-interrupt" fn invalid_tss(stack_frame: InterruptStackFrame, error_code: u64) {
        log::error!(
            "exception: 0xa invalid_tss, error_code={error_code:?}, stack_frame={stack_frame:#?}"
        );
        loop {}
    }

    pub extern "x86-interrupt" fn segment_not_present(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0xb segment_not_present, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn stack_segment_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0xc stack_segment_fault, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn general_protection_fault(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0xd general_protection_fault, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn page_fault(
        stack_frame: InterruptStackFrame,
        error_code: PageFaultErrorCode,
    ) {
        log::error!(
            "exception: 0xe page_fault, error_code={error_code:?}, stack_frame={stack_frame:#?}"
        );
        loop {}
    }

    pub extern "x86-interrupt" fn x87_floating_point(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x10 x87_floating_point, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn alignment_check(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0x11 alignment_check, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn machine_check(stack_frame: InterruptStackFrame) -> ! {
        panic!("exception: 0x12 machine_check, stack_frame={stack_frame:#?}");
    }

    pub extern "x86-interrupt" fn simd_floating_point(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x13 simd_floating_point, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn virtualization(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x14 virtualization, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn cp_protection_exception(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0x15 cp_protection_exception, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn hv_injection_exception(stack_frame: InterruptStackFrame) {
        log::error!("exception: 0x1c hv_injection_exception, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn vmm_communication_exception(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0x1d vmm_communication_exception, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }

    pub extern "x86-interrupt" fn security_exception(
        stack_frame: InterruptStackFrame,
        error_code: u64,
    ) {
        log::error!("exception: 0x1e security_exception, error_code={error_code:?}, stack_frame={stack_frame:#?}");
        loop {}
    }
}
