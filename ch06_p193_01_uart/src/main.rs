//! Wio TerminalのUART通信を0.6.1クレートで書き直そうとしたが、上手くいかなかった
//! 良い方法が見つかるまで作成中止
//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*;
use wio_terminal as wio; // 必須クレート // ほぼ必須モジュール
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

use core::fmt::Write;

// 絶対に初期化しないといけないので、いったんNoneで初期化する
static mut UART_USER: Option<
    atsamd_hal::sercom::uart::Uart<
        atsamd_hal::sercom::uart::Config<wio::UartPads>,
        atsamd_hal::sercom::uart::Duplex,
    >,
> = None;

#[wio_terminal::entry] // 必須アトリビュート
fn main() -> ! {
    // 初期化
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut sets: wio::Sets = wio::Pins::new(peripherals.PORT).split();
    let mut serial = sets.uart.init(
        &mut clocks,
        9600.hz(),
        peripherals.SERCOM2,
        &mut peripherals.MCLK,
    );
    

    // TODO: グローバル変数に格納されているNoneをSomeで上書きする
    unsafe {
        // None が格納されているので Someで上書き
        UART_USER = Some(serial);
        // オブジェクトの所有権をムーブしないように as_mut() で可変参照を入手する
        //let mut textbuffer: heapless::String<heapless::consts::U256> = heapless::String::new();
        //let version = "aaa";
        //writeln!(textbuffer, "fw: {}", version).unwrap();
        // TODO: 「this is UART example!」と出力する
        //writeln!(textbuffer, "this is {} example!", "UART").unwrap();

        //UART_USER.as_mut().unwrap().write_data(1_u32);
        //writeln!(UART_USER.as_mut().unwrap(), "hello {}", "world").unwrap();
    }

    // TODO: わざとNoneをunwrap()してパニックを発生させる
    //let noen: Option<usize> = None;
    //noen.unwrap();
    // ここまで 初期化

    // 組込みはloop必須
    loop {}
    // ここまでloop処理
}
// ここまでmain関数

// TODO: パニックハンドラを実装する
/*
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        UART_USER.as_mut().unwrap().write(2_u8).unwrap();

        //writeln!(UART_USER.as_mut().unwrap(), "panic: {}", info).ok();
    }
    loop {}
}

*/
