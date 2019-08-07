use alloc::boxed::Box;
use esp_idf_sys::{vTaskDelete, xTaskCreatePinnedToCore};

#[allow(non_upper_case_globals)]
const tskNO_AFFINITY: i32 = i32::max_value();

pub fn spawn<F>(func: F, name: &'static str, stack_depth: u32, priority: u32, affinity: Option<i32>)
where
    F: FnMut(),
{
    extern "C" fn wrapper(arg: *mut esp_idf_sys::std::os::raw::c_void) {
        let arg = arg as *mut Box<dyn FnMut()>;
        let mut func = unsafe { Box::from_raw(arg) };
        func();

        unsafe {
            // passing null ends the current task
            vTaskDelete(core::ptr::null_mut());
        }
    }

    let func: Box<dyn FnMut()> = Box::new(func);
    let func = Box::new(func);
    let func = Box::into_raw(func);
    unsafe {
        xTaskCreatePinnedToCore(
            Some(wrapper),
            name.as_ptr() as *const i8,
            stack_depth,
            func as *mut esp_idf_sys::std::os::raw::c_void,
            priority,
            core::ptr::null_mut(),
            affinity.unwrap_or(tskNO_AFFINITY),
        );
    }
}
