use crate::errors::{Error, EspError, Result};
use core::{convert::TryInto, marker::PhantomData, ptr};
use embedded_hal::{
    blocking::spi::{Transfer, Write},
    spi::{Mode, MODE_0, MODE_1, MODE_2, MODE_3},
};
use esp_idf_sys::{
    spi_bus_add_device, spi_bus_config_t, spi_bus_free, spi_bus_initialize, spi_bus_remove_device,
    spi_device_handle_t, spi_device_interface_config_t, spi_device_polling_transmit,
    spi_host_device_t, spi_host_device_t_HSPI_HOST, spi_host_device_t_VSPI_HOST, spi_transaction_t,
    spi_transaction_t__bindgen_ty_1, spi_transaction_t__bindgen_ty_2, std::os::raw::c_void, ESP_OK,
};

const NO_PIN: i32 = -1;

#[derive(Clone, Copy, Debug)]
pub enum Which {
    HSpi,
    VSpi,
}

impl From<Which> for spi_host_device_t {
    fn from(value: Which) -> Self {
        use Which::*;

        match value {
            HSpi => spi_host_device_t_HSPI_HOST,
            VSpi => spi_host_device_t_VSPI_HOST,
        }
    }
}

pub struct Master {
    which: Which,
}

impl Master {
    pub unsafe fn new(
        which: Which,
        mosi_pin: Option<i32>,
        miso_pin: Option<i32>,
        clk_pin: Option<i32>,
    ) -> Result<Self> {
        let bus_config = spi_bus_config_t {
            mosi_io_num: mosi_pin.unwrap_or(NO_PIN),
            miso_io_num: miso_pin.unwrap_or(NO_PIN),
            sclk_io_num: clk_pin.unwrap_or(NO_PIN),
            quadwp_io_num: NO_PIN,
            quadhd_io_num: NO_PIN,
            max_transfer_sz: 0, // use the default (4094)
            flags: 0,           // the default is fine
            intr_flags: 0,      // TODO
        };

        // TODO: dma channel number
        EspError(spi_bus_initialize(which.into(), &bus_config, 1)).into_result()?;

        Ok(Self { which })
    }
}

impl Drop for Master {
    fn drop(&mut self) {
        unsafe {
            let err = spi_bus_free(self.which.into());
            assert_eq!(err, ESP_OK as i32);
        }
    }
}

pub struct Device<'a> {
    device: spi_device_handle_t,
    bus: PhantomData<&'a Master>,
}

impl<'a> Device<'a> {
    pub unsafe fn new(
        bus: &'a Master,
        clock_speed_hz: u32,
        mode: Mode,
        cs_pin: Option<i32>,
        queue_size: usize,
    ) -> Result<Self> {
        let mode = match mode {
            MODE_0 => 0,
            MODE_1 => 1,
            MODE_2 => 2,
            MODE_3 => 3,
        };

        let device_config = spi_device_interface_config_t {
            command_bits: 0,
            address_bits: 0,
            dummy_bits: 0,
            mode,
            duty_cycle_pos: 0,
            cs_ena_pretrans: 0,
            cs_ena_posttrans: 0,
            clock_speed_hz: clock_speed_hz.try_into().unwrap(),
            input_delay_ns: 0,
            spics_io_num: cs_pin.unwrap_or(NO_PIN),
            flags: 0,
            queue_size: queue_size.try_into().unwrap(),
            pre_cb: None,
            post_cb: None,
        };

        let mut device = ptr::null_mut();
        EspError(spi_bus_add_device(
            bus.which.into(),
            &device_config,
            &mut device,
        ))
        .into_result()?;

        Ok(Self {
            device,
            bus: PhantomData,
        })
    }
}

impl<'a> Drop for Device<'a> {
    fn drop(&mut self) {
        unsafe {
            let err = spi_bus_remove_device(self.device);
            assert_eq!(err, ESP_OK as i32);
        }
    }
}

impl<'a> Write<u8> for Device<'a> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<()> {
        let mut transaction = spi_transaction_t {
            flags: 0,
            cmd: 0,
            addr: 0,
            length: words.len() * 8,
            rxlength: 0,
            user: ptr::null_mut(),
            __bindgen_anon_1: spi_transaction_t__bindgen_ty_1 {
                tx_buffer: words.as_ptr() as *const c_void,
            },
            __bindgen_anon_2: spi_transaction_t__bindgen_ty_2 {
                rx_buffer: ptr::null_mut(),
            },
        };

        unsafe {
            EspError(spi_device_polling_transmit(self.device, &mut transaction)).into_result()
        }
    }
}

impl<'a> Transfer<u8> for Device<'a> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8]> {
        let mut transaction = spi_transaction_t {
            flags: 0,
            cmd: 0,
            addr: 0,
            length: words.len() * 8,
            rxlength: 0,
            user: ptr::null_mut(),
            __bindgen_anon_1: spi_transaction_t__bindgen_ty_1 {
                tx_buffer: words.as_ptr() as *const c_void,
            },
            __bindgen_anon_2: spi_transaction_t__bindgen_ty_2 {
                rx_buffer: words.as_mut_ptr() as *mut c_void,
            },
        };

        unsafe {
            EspError(spi_device_polling_transmit(self.device, &mut transaction)).into_result()?;
        }

        Ok(words)
    }
}
