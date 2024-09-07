// jscc comes with builtins for dealing with MMIO (volatile pointers).
// In this example we will blink an LED connected to pin 5 of an Arduino UNO.

const LED_PIN = 5;
const DDRB = 0x04;
const PORTB = 0x05;

while (true) {
    outb(DDRB, 1 << LED_PIN);
    outb(PORTB, 1 << LED_PIN);
    outb(PORTB, 0);
    for (let i = 0; i < 100000; i++);
}