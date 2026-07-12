/*
 * Audio-Reactive LED
 *
 * Hardware:
 *   - Electret microphone module (MAX4466 / KY-037 / MAX9814)
 *     VCC → 5V (or 3.3V if module is 3.3V-only)
 *     GND → GND
 *     OUT → A0
 *
 *   - (Optional) External LED + 220Ω resistor on pin 9 for PWM brightness
 *     Or use the built-in LED on pin 13 (digital on/off only)
 *
 * Reads audio amplitude from the mic, then lights the LED in sync with the beat.
 */

#define MIC_PIN    A0       // Microphone analog input
#define LED_PIN    13       // Built-in LED (digital on/off)
#define PWM_PIN    9        // Optional: external LED on pin 9 for brightness

#define SAMPLES    64       // Samples per reading cycle
#define THRESHOLD  30       // Sensitivity — lower = more reactive

void setup() {
  pinMode(LED_PIN, OUTPUT);
  pinMode(PWM_PIN, OUTPUT);
  Serial.begin(9600);       // Optional: view readings in Serial Monitor
}

void loop() {
  // --- Sample the microphone ---
  int maxVal = 0;
  int minVal = 1023;

  for (int i = 0; i < SAMPLES; i++) {
    int val = analogRead(MIC_PIN);
    if (val > maxVal) maxVal = val;
    if (val < minVal) minVal = val;
    delayMicroseconds(250);  // ~4 kHz sample rate
  }

  // --- Calculate amplitude (peak-to-peak) ---
  int amplitude = maxVal - minVal;
  Serial.println(amplitude); // View in Serial Plotter (Tools → Serial Plotter)

  // --- Built-in LED: on/off based on threshold ---
  if (amplitude > THRESHOLD) {
    digitalWrite(LED_PIN, HIGH);
  } else {
    digitalWrite(LED_PIN, LOW);
  }

  // --- External PWM LED: brightness follows amplitude ---
  // Map amplitude (0-1023) to PWM brightness (0-255)
  int brightness = map(amplitude, 0, 200, 0, 255);
  brightness = constrain(brightness, 0, 255);
  analogWrite(PWM_PIN, brightness);
}
