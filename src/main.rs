#![no_std]
#![no_main]

use lazy_static::lazy_static;
use spin::Mutex;
use bare_metal_tracer::TracerGame;
use pluggable_interrupt_os::HandlerTable;

use pc_keyboard::DecodedKey;

lazy_static! {
    static ref TRACER: Mutex<TracerGame> = Mutex::new(TracerGame::new());
}

fn tick() {
    TRACER.lock().tick();
}

fn key(key: DecodedKey) {
    TRACER.lock().key(key);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .start()
}
