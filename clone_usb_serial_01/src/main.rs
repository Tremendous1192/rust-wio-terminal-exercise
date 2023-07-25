//! Windows 10 だと動かない
#![no_std]
#![no_main]

use panic_halt as _; // 必須クレート
use wio_terminal as wio; // 必須クレート

use wio::pac::interrupt; // 割り込み処理の宣言 use を外すことができない
use wio::prelude::*; // ほぼ必須モジュール

use eg::mono_font::ascii::FONT_6X12; // フォント
use eg::prelude::*; // 描画のほぼ必須モジュール
use embedded_graphics as eg; // 描画関係

use heapless::spsc::Queue; // 送受信処理の宣言 use を外すことができない

#[wio::entry]
fn main() -> ! {
    // 必須インスタンス
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let mut core = wio::pac::CorePeripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);
    let sets = wio::Pins::new(peripherals.PORT).split();

    // 通信が成功したときに内臓LEDを点滅させたい
    let mut user_led = sets.user_led.into_push_pull_output();
    user_led.set_low().unwrap();

    // LCDディスプレイの初期化
    let (display, _backlight) = sets
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM7,
            &mut peripherals.MCLK,
            58.mhz(),
            &mut delay,
        )
        .unwrap();

    // 文字描画インスタンス
    let mut t = Terminal::new(display);
    t.write_str("Hello! Send text to me over the USB serial port, and I'll display it!");
    t.write_str("\n");
    t.write_str("On linux:\n");
    t.write_str("  sudo stty -F /dev/ttyACM0 115200 raw -echo\n");
    t.write_str("  sudo bash -c \"echo 'Hi' > /dev/ttyACM0\"\n");

    // USB制御インスタンスの初期化
    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(sets.usb.usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.MCLK,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };
    unsafe {
        USB_SERIAL = Some(usbd_serial::SerialPort::new(bus_allocator));
        USB_BUS = Some(
            usb_device::prelude::UsbDeviceBuilder::new(
                bus_allocator,
                usb_device::prelude::UsbVidPid(0x16c0, 0x27dd),
            )
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build(),
        );
    }
    unsafe {
        core.NVIC.set_priority(interrupt::USB_OTHER, 1);
        core.NVIC.set_priority(interrupt::USB_TRCPT0, 1);
        core.NVIC.set_priority(interrupt::USB_TRCPT1, 1);
        cortex_m::peripheral::NVIC::unmask(interrupt::USB_OTHER);
        cortex_m::peripheral::NVIC::unmask(interrupt::USB_TRCPT0);
        cortex_m::peripheral::NVIC::unmask(interrupt::USB_TRCPT1);
    }

    // 授受データの受信側(.1)のインスタンス化
    let mut consumer = unsafe { Q.split().1 };
    loop {
        // 通信が来たときに文字を描画してLEDを点滅させる
        if let Some(segment) = consumer.dequeue() {
            t.write(segment);
            user_led.toggle().ok();
        }
    }
}

// 32文字までの入力のバッファー(?)
type TextSegment = ([u8; 32], usize);

// 文字描画構造体
struct Terminal<'a> {
    text_style: eg::mono_font::MonoTextStyle<'a, eg::pixelcolor::Rgb565>,
    cursor: eg::prelude::Point,
    display: wio::LCD,
    scroller: wio::Scroller,
}

impl<'a> Terminal<'a> {
    // 初期化
    pub fn new(mut display: wio::LCD) -> Self {
        // Clear the screen.
        let style = eg::primitives::PrimitiveStyleBuilder::new()
            .fill_color(eg::pixelcolor::Rgb565::BLACK)
            .build();
        let backdrop = eg::primitives::Rectangle::with_corners(
            eg::prelude::Point::new(0, 0),
            eg::prelude::Point::new(320, 320),
        )
        .into_styled(style);
        backdrop.draw(&mut display).ok().unwrap();

        // 文字の折り返し(?)
        let scroller = display.configure_vertical_scroll(0, 0).unwrap();

        Self {
            text_style: eg::mono_font::MonoTextStyle::new(
                &FONT_6X12,
                eg::pixelcolor::Rgb565::WHITE,
            ),
            cursor: eg::prelude::Point::new(10, 15), // 文字が画面内に収まるように初期位置を調整
            display,
            scroller,
        }
    }

    // 文字列を描画する
    pub fn write_str(&mut self, str: &str) {
        for character in str.chars() {
            self.write_character(character);
        }
    }

    // 文字を描画する
    pub fn write_character(&mut self, c: char) {
        if self.cursor.x >= 320 || c == '\n' {
            self.cursor = Point::new(0, self.cursor.y + FONT_6X12.character_size.height as i32);
        }
        if self.cursor.y >= 240 {
            self.animate_clear();
            self.cursor = Point::new(0, 0);
        }

        if c != '\n' {
            let mut buf = [0u8; 8];
            eg::text::Text::new(c.encode_utf8(&mut buf), self.cursor, self.text_style)
                .draw(&mut self.display)
                .ok()
                .unwrap();

            self.cursor.x += (FONT_6X12.character_size.width + FONT_6X12.character_spacing) as i32;
        }
    }

    // 描画
    pub fn write(&mut self, segment: TextSegment) {
        let (buf, count) = segment;
        for (i, character) in buf.iter().enumerate() {
            if i >= count {
                break;
            }
            self.write_character(*character as char);
        }
    }

    // 画面の初期化(?)
    fn animate_clear(&mut self) {
        for x in (0..320).step_by(FONT_6X12.character_size.width as usize) {
            self.display
                .scroll_vertically(&mut self.scroller, FONT_6X12.character_size.width as u16)
                .ok()
                .unwrap();
            eg::primitives::Rectangle::with_corners(
                Point::new(x, 0),
                Point::new(x + FONT_6X12.character_size.width as i32, 240),
            )
            .into_styled(
                eg::primitives::PrimitiveStyleBuilder::new()
                    .fill_color(eg::pixelcolor::Rgb565::BLACK)
                    .build(),
            )
            .draw(&mut self.display)
            .ok()
            .unwrap();

            for _ in 0..1000 {
                cortex_m::asm::nop();
            }
        }
    }
}

// USB通信のstaticインスタンス
static mut USB_ALLOCATOR: Option<usb_device::bus::UsbBusAllocator<wio::hal::usb::UsbBus>> = None;
static mut USB_BUS: Option<usb_device::prelude::UsbDevice<wio::hal::usb::UsbBus>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<wio::hal::usb::UsbBus>> = None;
// 通信データのメモリ保存場所
static mut Q: heapless::spsc::Queue<TextSegment, 16_usize> = Queue::new();

// シリアル通信の受信データをWio Terminalの画面に表示する
fn poll_usb() {
    unsafe {
        // USB接続判定が生きている
        if let Some(usb_dev) = USB_BUS.as_mut() {
            // USB接続機器が生きている
            if let Some(serial) = USB_SERIAL.as_mut() {
                // データを受信できたかどうか判定する(?)
                usb_dev.poll(&mut [serial]);
                let mut buf = [0u8; 32];
                // 授受データの内、producer(.0)データを取り出す
                let mut terminal = Q.split().0;

                // ポートからデータを読み込めるかどうか
                if let Ok(count) = serial.read(&mut buf) {
                    // Queueテーブルの末尾に受信したデータを加える
                    terminal.enqueue((buf, count)).ok().unwrap();
                };
            }
        }
    };
}

// 3つのUSBポート(?)のいずれかで通信ができれば、画面に表示するために interrupt を宣言する
#[interrupt]
fn USB_OTHER() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}
