//! 組込みRustのおまじない
#![no_std] // 必須アトリビュート
#![no_main] // 必須アトリビュート
use panic_halt as _; // 必須クレート
use wio::prelude::*; // ほぼ必須
use wio_terminal as wio; // 必須クレート
const DISPLAY_WIDTH: u32 = 320; // 画面幅の定数. ほぼ必須で良いだろう
const DISPLAY_HEIGHT: u32 = 240; // 画面高さの定数. ほぼ必須で良いだろう

use eg::prelude::*; // 役割が多いのでとりあえずインポートしておく
use embedded_graphics as eg; // 描画関係

use itoa; // 整数型を文字列スライスに変換する

#[wio::entry]
fn main() -> ! {
    // 初期化
    // 必須
    let mut peripherals = wio::pac::Peripherals::take().unwrap();
    let core = wio::pac::CorePeripherals::take().unwrap();
    let mut clocks = wio::hal::clock::GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut sets = wio::Pins::new(peripherals.PORT).split();
    let mut delay = wio::hal::delay::Delay::new(core.SYST, &mut clocks);

    // 加速度センサドライバ
    let mut accel =
        sets.accelerometer
            .init(&mut clocks, peripherals.SERCOM4, &mut peripherals.MCLK);

    // ディスプレイドライバ
    let (mut display, _backlight) = sets
        .display
        .init(
            &mut clocks,
            peripherals.SERCOM7,
            &mut peripherals.MCLK,
            60_u32.mhz(),
            &mut delay,
        )
        .unwrap();

    // 背景
    let background = make_black_rectangle(0_i32, 0_i32, DISPLAY_WIDTH, DISPLAY_HEIGHT);
    background.draw(&mut display).unwrap();

    // 前回の入力値を塗りつぶして見えなくする黒い四角形
    let refresher = make_black_rectangle(30_i32, 20_i32, 50_u32, 70_u32);

    // 説明文を表示する
    let style = eg::mono_font::MonoTextStyle::new(
        &eg::mono_font::ascii::FONT_10X20,
        eg::pixelcolor::Rgb565::WHITE,
    );
    eg::text::Text::new(
        "Input value of light senser",
        eg::prelude::Point::new(15_i32, 15_i32),
        style,
    )
    .draw(&mut display)
    .unwrap();

    // 各成分の値(リフレッシュは不要)
    eg::text::Text::new("x: ", eg::prelude::Point::new(15_i32, 35_i32), style)
        .draw(&mut display)
        .unwrap();
    eg::text::Text::new("y: ", eg::prelude::Point::new(15_i32, 50_i32), style)
        .draw(&mut display)
        .unwrap();
    eg::text::Text::new("z: ", eg::prelude::Point::new(15_i32, 65_i32), style)
        .draw(&mut display)
        .unwrap();

    // 1秒ごとに加速度センサの入力値を画面に表示する
    loop {
        // 前回の入力値を塗りつぶす
        refresher.draw(&mut display).unwrap();

        // 加速度センサの入力値をstr型に変換する
        let accelerometer::vector::F32x3 { x, y, z } = accel.accel_norm().unwrap();

        // センサの入力値を画面に表示する
        eg::text::Text::new(
            itoa::Buffer::new().format((x * 100.0) as u32),
            eg::prelude::Point::new(40_i32, 35_i32),
            style,
        )
        .draw(&mut display)
        .unwrap();

        eg::text::Text::new(
            itoa::Buffer::new().format((y * 100.0) as u32),
            eg::prelude::Point::new(40_i32, 50_i32),
            style,
        )
        .draw(&mut display)
        .unwrap();

        eg::text::Text::new(
            itoa::Buffer::new().format((z * 100.0) as u32),
            eg::prelude::Point::new(40_i32, 65_i32),
            style,
        )
        .draw(&mut display)
        .unwrap();

        delay.delay_ms(1_000_u16);
    }
}

// 黒い四角形を作成する
fn make_black_rectangle(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> embedded_graphics::primitives::Styled<
    embedded_graphics::primitives::Rectangle,
    embedded_graphics::primitives::PrimitiveStyle<embedded_graphics::pixelcolor::Rgb565>,
> {
    let style = eg::primitives::PrimitiveStyleBuilder::new()
        .fill_color(eg::pixelcolor::Rgb565::BLACK)
        .build();
    let rectamgle = eg::primitives::Rectangle::new(
        eg::prelude::Point::new(x, y),
        eg::prelude::Size::new(width, height),
    )
    .into_styled(style);
    rectamgle
}
