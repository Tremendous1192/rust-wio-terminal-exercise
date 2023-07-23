# clone_usb_serial_01
[USB シリアル通信のサンプルプログラム](https://github.com/atsamd-rs/atsamd/blob/e241dc02032812acfcb606ee87ea8e625243b95a/boards/wio_terminal/examples/usb_serial_display.rs)の写経とクレートを比較的新しい版に変更。

wio-terminalクレート 0.6.1の[コミット番号 e24dc0203](https://github.com/atsamd-rs/atsamd/tree/e241dc02032812acfcb606ee87ea8e625243b95a)のプログラムを引用した。

# Windows 10 だと動作しない
原因が分からない。

初期化時の文字列に Linux の言及があったので、対象外の可能性がある。

/// Makes the wio_terminal appear as a USB serial port. The screen can
/// be written to by sending messages down the serial port.
