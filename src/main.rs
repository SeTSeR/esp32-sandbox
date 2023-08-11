#![no_std]
#![no_main]

use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData, WorkResult},
    gatt, Ble, HciConnector,
};
use esp_backtrace as _;
use esp_println::{logger::init_logger, println};
use esp_wifi::{ble::controller::BleConnector, initialize, EspWifiInitFor};
use hal::{
    clock::{ClockControl, CpuClock},
    peripherals::*,
    prelude::*,
    timer::TimerGroup,
    Rng, Rtc, IO,
};

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = peripherals.DPORT.split();
    let mut peripheral_clock_control = system.peripheral_clock_control;
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();

    let timer = TimerGroup::new(peripherals.TIMG1, &clocks, &mut peripheral_clock_control).timer0;
    let init = initialize(
        EspWifiInitFor::Ble,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let button = io.pins.gpio0.into_pull_down_input();
    let pin = io.pins.gpio23.into_pull_down_input();

    let mut debounce_cnt = 500;

    let (_, mut bluetooth) = peripherals.RADIO.split();

    loop {
        let connector = BleConnector::new(&init, &mut bluetooth);
        let hci = HciConnector::new(connector, esp_wifi::current_millis);
        let mut ble = Ble::new(&hci);

        println!("{:?}", ble.init());
        println!("{:?}", ble.cmd_set_le_advertising_parameters());
        println!(
            "{:?}",
            ble.cmd_set_le_advertising_data(
                create_advertising_data(&[
                    AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                    AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                    AdStructure::CompleteLocalName("ESP32"),
                ])
                .unwrap()
            )
        );
        println!("{:?}", ble.cmd_set_le_advertise_enable(true));

        println!("started advertising");

        let mut rf = |_offset: usize, data: &mut [u8]| {
            data[..20].copy_from_slice(&b"Hello Bare-Metal BLE"[..]);
            17
        };
        let mut wf = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:x?}", offset, data);
        };

        let mut wf2 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:x?}", offset, data);
        };

        let mut rf3 = |_offset: usize, data: &mut [u8]| {
	    if pin.is_high().unwrap() {
		data[..15].copy_from_slice(&b"motion detected"[..]);
		15
	    } else {
		data[..10].copy_from_slice(&b"no motion"[..]);
		10
	    }
        };
        let mut wf3 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: Offset {}, data {:x?}", offset, data);
        };

        gatt!([service {
            uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
            characteristics: [
                characteristic {
                    uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
                    read: rf,
                    write: wf,
                },
                characteristic {
                    uuid: "957312e0-2354-11eb-9f10-fbc30a62cf38",
                    write: wf2,
                },
                characteristic {
                    name: "my_characteristic",
                    uuid: "987312e0-2354-11eb-9f10-fbc30a62cf38",
                    notify: true,
                    read: rf3,
                    write: wf3,
                },
            ],
        },]);

        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes);

        loop {
            let mut notification = None;

            if button.is_low().unwrap() && debounce_cnt > 0 {
                debounce_cnt -= 1;
                if debounce_cnt == 0 {
                    let mut cccd = [0u8; 1];
                    if let Some(1) = srv.get_characteristic_value(
                        my_characteristic_notify_enable_handle,
                        0,
                        &mut cccd,
                    ) {
                        // if notifications enabled
                        if cccd[0] == 1 {
                            notification = Some(NotificationData::new(
                                my_characteristic_handle,
                                &b"Notification"[..],
                            ));
                        }
                    }
                }
            };

            if button.is_high().unwrap() {
                debounce_cnt = 500;
            }

            match srv.do_work_with_notification(notification) {
                Ok(res) => {
                    if let WorkResult::GotDisconnected = res {
                        break;
                    }
                }
                Err(err) => {
                    println!("{:x?}", err);
                }
            }
        }
    }
}
