# esp32c3-fm

使用esp32c3和rda5807芯片实现的fm收音机

## 硬件

硬件开源地址： https://oshwhub.com/intentz/shou-yin-ji-v1

- esp32c3
- rda5807m
- 按钮x4
- ssd1306 oled显示屏
- ec11旋钮编码器
- 功放x2

## 软件

编译
```shell
cargo build --release --bin rda5807m_demo
```

编译并烧写

```shell
cargo run --release --bin rda5807m_demo
```