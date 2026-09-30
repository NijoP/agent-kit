# EAK V1 Authoritative 3,500-Component Library Taxonomy

**Version:** 1.0
**Status:** ACTIVE
**Date:** 2025-08-20
**Owner:** Terminal 1 (Library Taxonomy)

---

## Executive Summary

This document defines the authoritative component taxonomy for EAK V1: **3,500 components** allocated across 16 component families, supporting the "Intent In. Manufactured Board Out." vision. The taxonomy distinguishes between **500 existing verified passives** (Day 1 foundation) and **3,000 planned additional components** across all required families for golden-board demonstration.

| Metric | Value |
|--------|-------|
| **Total Components** | 3,500 |
| **Existing Verified (Day 1)** | 500 (Passives only) |
| **Planned Additional** | 3,000 |
| **Component Families** | 16 |
| **Status Model** | PLANNED / VERIFIED / AVAILABLE / BLOCKED / NOT_FOUND |

---

## 1. Component Family Allocation Matrix

### 1.1 Allocation Summary (3,500 Total)

| Family | Current (Verified) | Additional (Planned) | Total | % of Library | Status |
|--------|-------------------|---------------------|-------|--------------|--------|
| **Passives** | 500 | 175 | **675** | 19.3% | VERIFIED/AVAILABLE |
| **Diodes** | 0 | 205 | **205** | 5.9% | PLANNED |
| **Transistors** | 0 | 185 | **185** | 5.3% | PLANNED |
| **Analog ICs / Op-Amps** | 0 | 285 | **285** | 8.1% | PLANNED |
| **Power Management** | 0 | 325 | **325** | 9.3% | PLANNED |
| **Digital Logic** | 0 | 225 | **225** | 6.4% | PLANNED |
| **MCUs** | 0 | 285 | **285** | 8.1% | PLANNED |
| **Memory** | 0 | 185 | **185** | 5.3% | PLANNED |
| **Communication** | 0 | 235 | **235** | 6.7% | PLANNED |
| **Sensors** | 0 | 185 | **185** | 5.3% | PLANNED |
| **RF / Wireless** | 0 | 135 | **135** | 3.9% | PLANNED |
| **Audio** | 0 | 85 | **85** | 2.4% | PLANNED |
| **Protection** | 0 | 135 | **135** | 3.9% | PLANNED |
| **Connectors** | 0 | 185 | **185** | 5.3% | PLANNED |
| **Electromechanical** | 0 | 85 | **85** | 2.4% | PLANNED |
| **Specialized** | 0 | 85 | **85** | 2.4% | PLANNED |
| **TOTAL** | **500** | **3,000** | **3,500** | **100%** | — |

### 1.2 Allocation Validation

```
Current Verified:     500
Planned Additional: 3,000
────────────────────────
Total:             3,500  ✓ MATCHES REQUIREMENT
```

**Arithmetic Check:**
- 500 + 3,000 = 3,500 ✓
- Sum of family totals = 675+205+185+285+325+225+285+185+235+185+135+85+135+185+85+85 = 3,500 ✓
- No negative counts ✓
- No duplicate families ✓
- All 16 required families represented ✓

---

## 2. Family Definitions & Subfamily Breakdown

### 2.1 Passives (675 total: 500 verified + 175 planned)

**Engineering Relevance:** Foundation of every PCB — decoupling, filtering, termination, timing, current sensing, energy storage.

| Subfamily | Current | Planned | Total | Key Engineering Parameters |
|-----------|---------|---------|-------|---------------------------|
| Resistors (Thin/Thick Film, Wirewound, Metal Foil) | 180 | 43 | 223 | Resistance, Tolerance, TCR, Power, Voltage Rating, Noise, Pulse Withstand |
| Capacitors (MLCC, Electrolytic, Tantalum, Film, Polymer) | 150 | 42 | 192 | Capacitance, Voltage, ESR, ESL, Dielectric, Temperature Coeff, DC Bias Derating |
| Inductors (Power, Signal, Coupled, Bead) | 80 | 28 | 108 | Inductance, DCR, Saturation Current, RMS Current, SRF, Q Factor, Core Material |
| Ferrite Beads / EMI Suppression | 40 | 19 | 59 | Impedance @ 100MHz, DC Resistance, Rated Current |
| Crystals / Oscillators / Resonators | 30 | 17 | 47 | Frequency, Load Capacitance, ESR, Drive Level, Stability, Aging |
| Transformers (Signal, Power, Isolation) | 20 | 11 | 31 | Turns Ratio, Inductance, Leakage L, DCR, Isolation Voltage, Power |
| Thermistors (NTC/PTC) | 0 | 10 | 10 | Resistance @ 25°C, B-Constant, Dissipation Constant, Thermal Time Constant |
| Varistors / MOVs | 0 | 5 | 5 | Varistor Voltage, Peak Current, Energy Rating, Capacitance |

**Status Distribution:**
- VERIFIED: 500 (existing Day 1 foundation)
- AVAILABLE: 115 (common jellybean parts with verified assets)
- PLANNED: 60 (specialized/high-voltage/high-power variants)

**Golden-Board Relevance:** ESSENTIAL — Every board requires passives for power integrity, signal integrity, EMI, timing.

---

### 2.2 Diodes (205 total)

**Engineering Relevance:** Rectification, protection, switching, voltage reference, RF detection, lighting.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Rectifier (Standard, Fast, Ultra-Fast, Schottky) | 49 | Vf, Vrrm, If, Ir, Trr, Package Thermal |
| Schottky (Power, Signal, SiC) | 33 | Vf (low), Vrrm, If, Junction Capacitance |
| Zener / Voltage Reference | 29 | Vz, Zzt, Zzk, Temperature Coefficient, Power |
| TVS / ESD Protection (Uni/Bi-directional) | 24 | Vrwm, Vbr, Ipp, Clamping Voltage, Capacitance |
| Switching / Small Signal (1N4148, BAS16, BAV99) | 20 | Trr, Cd, Vr, If |
| LED (Indication, Illumination, UV, IR) | 21 | Vf, Wavelength, Luminous Intensity, Viewing Angle, If |
| Photodiodes / Phototransistors | 13 | Responsivity, Dark Current, Capacitance, Bandwidth |
| Bridge Rectifiers | 8 | Vrrm, If, Vf, Package |
| Specialized (Gunn, PIN, Varactor, Tunnel, Avalanche) | 8 | Application-specific parameters |

**Status Distribution:**
- PLANNED: 205 (no existing verified diodes)

**Golden-Board Relevance:** ESSENTIAL — Power rectification, reverse polarity protection, ESD clamping, status LEDs.

---

### 2.3 Transistors (185 total)

**Engineering Relevance:** Switching, amplification, power conversion, logic level translation, motor drive.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| MOSFETs — N-Channel (Power, Logic-Level, Trench) | 55 | Vds, Id, Rds(on), Vgs(th), Qg, Ciss, Coss, Thermal |
| MOSFETs — P-Channel | 28 | Vds, Id, Rds(on), Vgs(th), Qg |
| BJTs — NPN/PNP (Small Signal, Power, Darlington) | 32 | Vceo, Ic, hFE, Vce(sat), ft, Power |
| IGBTs | 15 | Vces, Ic, Vge(th), Vce(sat), Eon/Eoff, Thermal |
| GaN / SiC FETs | 10 | Vds, Id, Rds(on), Qg, Reverse Recovery, Thermal |
| JFETs | 10 | Vgs(off), Idss, gm, Ciss, Noise |
| RF Transistors (HEMT, LDMOS) | 15 | Frequency Range, Gain, P1dB, PAE, Voltage |
| Digital Transistors (Pre-biased, RET) | 20 | Vceo, Ic, R1/R2 Ratio, Switching Time |

**Status Distribution:**
- PLANNED: 185

**Golden-Board Relevance:** HIGH — Power path switching, load switches, motor drivers, level translation.

---

### 2.4 Analog ICs / Op-Amps (285 total)

**Engineering Relevance:** Signal conditioning, filtering, amplification, precision measurement, power monitoring.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| General-Purpose Op-Amps | 48 | GBW, Slew Rate, Vos, Ib, Iq, Voltage Range, Rail-to-Rail |
| Precision / Low-Offset Op-Amps | 38 | Vos (µV), TCVos, 1/f Noise, CMRR, PSRR |
| High-Speed / Wideband Op-Amps | 29 | GBW (>100MHz), Slew Rate, Settling Time, Distortion |
| Low-Power / Micropower Op-Amps | 23 | Iq (<10µA), GBW, Voltage Range |
| Instrumentation Amplifiers | 23 | Gain Accuracy, CMRR, Vos, Bandwidth, Input Impedance |
| Comparators (Open-Drain, Push-Pull, Rail-to-Rail) | 28 | Propagation Delay, Overdrive, Hysteresis, Output Type |
| Voltage References (Series, Shunt, Programmable) | 23 | Voltage, Initial Accuracy, TC, Noise, Load Regulation |
| ADCs / DACs (SAR, Delta-Sigma, Pipeline, R-2R) | 33 | Resolution, Sample Rate, INL/DNL, SNR, THD, Interface |
| Analog Switches / Multiplexers | 19 | Ron, Ron Flatness, Charge Injection, Bandwidth, Voltage |
| Specialized (Log Amps, RMS-DC, VCA, Temp Sensors) | 21 | Application-specific |

**Status Distribution:**
- PLANNED: 285

**Golden-Board Relevance:** HIGH — Sensor front-ends, power monitoring, audio, precision measurement.

---

### 2.5 Power Management (325 total)

**Engineering Relevance:** Voltage regulation, battery management, power sequencing, protection — critical for power integrity.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Buck Converters (Sync, Async, Multi-Phase) | 56 | Vin, Vout, Iout, Frequency, Efficiency, Control Mode |
| Boost / Buck-Boost / SEPIC | 33 | Vin Range, Vout, Iout, Switch Current Limit |
| LDO Regulators (Standard, Low-Noise, High-PSRR) | 46 | Vin, Vout, Iout, Dropout, Noise, PSRR, Iq |
| PMICs (Multi-Rail, Sequenced) | 28 | Rail Count, Sequencing, I2C Control, GPIOs |
| Battery Management (Charger, Fuel Gauge, Protection) | 37 | Chemistry, Charge Current, Voltage Accuracy, Safety |
| Load Switches / eFuses / Ideal Diodes | 32 | Current Limit, Ron, Response Time, Reverse Blocking |
| Supervisor / Reset / Watchdog | 28 | Threshold, Timeout, Watchdog Window, Output Type |
| Hot-Swap / Inrush Control | 19 | Current Limit, dV/dt, Fault Response, Reporting |
| LED Drivers (Constant Current, Matrix, RGB) | 23 | Topology, Current Accuracy, Dimming, Channels |
| PoE / PD Controllers | 14 | Class, Power, Efficiency, Aux Supply |
| DC-DC Controllers (External FET) | 9 | Phases, Current Sense, Control Loop, Sync |

**Status Distribution:**
- PLANNED: 325

**Golden-Board Relevance:** ESSENTIAL — Every board needs power conversion, sequencing, protection.

---

### 2.6 Digital Logic (225 total)

**Engineering Relevance:** Glue logic, level translation, bus buffering, clock distribution, signal conditioning.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Gates (AND, OR, NAND, NOR, XOR, NOT) | 38 | Propagation Delay, Drive Strength, Voltage Range |
| Buffers / Line Drivers / Transceivers | 32 | Drive Current, Slew Rate, Enable, Bus Hold |
| Level Translators (Auto-Direction, Direction-Controlled) | 27 | Voltage Ranges, Propagation Delay, Channels |
| Flip-Flops / Latches / Registers | 27 | Clock Frequency, Setup/Hold, Output Drive |
| Counters / Shift Registers | 19 | Count Range, Serial/Parallel, Output Type |
| Decoders / Encoders / Muxes | 19 | Channels, Enable, Propagation Delay |
| Bus Switches / Signal Switches | 21 | Ron, Bandwidth, Charge Injection, Voltage |
| Clock Buffers / PLLs / Frequency Synthesizers | 21 | Jitter, Frequency Range, Output Count, Format |
| Specialized (Delay Lines, Parity, CRC) | 21 | Application-specific |

**Status Distribution:**
- PLANNED: 225

**Golden-Board Relevance:** HIGH — Level translation, clock distribution, bus buffering, glue logic.

---

### 2.7 MCUs (285 total)

**Engineering Relevance:** Central control, real-time processing, peripheral integration, connectivity.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| ARM Cortex-M0/M0+ (Ultra-Low Power) | 34 | Core, Frequency, Flash/RAM, Peripherals, Package |
| ARM Cortex-M3/M4/M4F (Mainstream) | 58 | Core, FPU, DSP, Frequency, Flash/RAM, Security |
| ARM Cortex-M7/M33/M35P (High Performance) | 34 | Core, Cache, MPU/TrustZone, DSP, Frequency |
| RISC-V (32/64-bit, Linux-Capable) | 29 | ISA Extensions, Frequency, Memory, Security |
| 8-bit (AVR, PIC, 8051, STM8) | 24 | Core, Frequency, Flash/RAM, Peripherals |
| 16-bit (MSP430, PIC24, RL78) | 20 | Core, Frequency, Ultra-Low Power, Peripherals |
| Wireless MCUs (BLE, WiFi, Thread, Zigbee, LoRa) | 43 | Protocol, TX/RX Power, Sensitivity, Coexistence |
| Automotive / Safety (ISO 26262, AEC-Q100) | 24 | Safety Level, ECC, Lockstep, Diagnostics |
| Specialized (Motor Control, DSP-Enhanced, Secure) | 19 | Peripherals, Accelerators, Security Features |

**Status Distribution:**
- PLANNED: 285

**Golden-Board Relevance:** ESSENTIAL — Main controller, wireless connectivity, real-time control.

---

### 2.8 Memory (185 total)

**Engineering Relevance:** Program storage, data logging, cache, configuration, boot.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| SPI NOR Flash (Boot, Code Storage) | 32 | Density, Frequency, Voltage, Quad/Octal SPI, Erase |
| SPI / I2C EEPROM | 28 | Density, Write Cycles, Page Size, Voltage, Speed |
| Parallel NOR / NAND Flash | 14 | Density, Bus Width, Voltage, ECC, Wear Leveling |
| SRAM (Async, Sync, Battery-Backed) | 23 | Density, Speed, Voltage, Retention, Interface |
| FRAM / MRAM / ReRAM | 14 | Density, Endurance, Speed, Voltage, Non-Volatile |
| SD / eMMC / UFS Controllers + Memory | 19 | Capacity, Speed Class, Interface, Voltage |
| Serial RAM / PSRAM | 19 | Density, Speed, Voltage, Burst, Latency |
| OTP / EFUSE / Secure Element | 19 | Security Features, Provisioning, Crypto, Interface |
| Specialized (NVRAM, Battery-Backed, Radiation-Hard) | 17 | Application-specific |

**Status Distribution:**
- PLANNED: 185

**Golden-Board Relevance:** HIGH — Boot flash, data logging, config storage, secure key storage.

---

### 2.9 Communication (235 total)

**Engineering Relevance:** Wired/wireless connectivity, protocols, isolation, signal integrity.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| UART / USART / UART Bridges | 24 | Baud Rate, Flow Control, FIFO, Voltage, Isolation |
| SPI / I2C / I3C Controllers / Expanders | 33 | Speed, Voltage Translation, Channels, DMA |
| USB (FS/HS/SS, PHY, PD Controllers) | 29 | Speed, PHY Integration, PD Role, Channels |
| Ethernet (MAC, PHY, Switch, TSN) | 23 | Speed, Interface (RGMII/SGMII), TSN, PoE |
| CAN / CAN-FD / LIN Transceivers & Controllers | 23 | Speed, Fault Tolerance, Isolation, Sleep Modes |
| RS-485 / RS-422 / RS-232 Transceivers | 18 | Speed, Isolation, ESD, Slew Rate, Nodes |
| PCIe / SATA / NVMe Controllers | 14 | Lanes, Gen, Power Management, SR-IOV |
| Industrial (EtherCAT, PROFINET, CC-Link) | 14 | Protocol, Cycle Time, Sync, Diagnostics |
| Specialized (1-Wire, SMBus, PMBus, MDIO) | 14 | Protocol, Addressing, Speed, Features |
| Optical (SFP/QSFP, Transimpedance Amps) | 14 | Data Rate, Reach, Wavelength, Power |
| Isolation (Digital, Power, Signal) | 29 | Channels, Data Rate, Isolation Voltage, CMTI |

**Status Distribution:**
- PLANNED: 235

**Golden-Board Relevance:** ESSENTIAL — External interfaces, fieldbus, USB, Ethernet, isolation.

---

### 2.10 Sensors (185 total)

**Engineering Relevance:** Environmental monitoring, motion, position, current/voltage/power sensing.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Temperature (Digital, Analog, RTD, Thermocouple) | 28 | Range, Accuracy, Resolution, Interface, Response Time |
| Pressure (Absolute, Gauge, Differential) | 18 | Range, Accuracy, Overpressure, Media Compatibility |
| Inertial (Accel, Gyro, IMU, 6/9-DoF) | 28 | Range, Sensitivity, Noise, Bandwidth, FIFO, Interface |
| Magnetic (Hall, Magnetometer, Angle, Current) | 24 | Range, Sensitivity, Angle Error, Bandwidth, Interface |
| Optical (Ambient, Proximity, ToF, Gesture, Color) | 18 | Range, Resolution, Wavelength, Interface, FoV |
| Current Sense (Shunt, Hall, Fluxgate, Rogowski) | 18 | Range, Accuracy, Bandwidth, Isolation, Directionality |
| Voltage / Power Monitors | 14 | Channels, Accuracy, Range, Alert, Interface |
| Environmental (Humidity, Gas, Air Quality, Particulate) | 18 | Analytes, Range, Accuracy, Lifetime, Interface |
| Position / Encoder (Rotary, Linear, Inductive) | 10 | Resolution, Accuracy, Speed, Interface, Absolute/Inc |
| Specialized (Flow, Force, Torque, Ultrasonic, Radar) | 9 | Application-specific |

**Status Distribution:**
- PLANNED: 185

**Golden-Board Relevance:** HIGH — System monitoring, motor control, environmental, power measurement.

---

### 2.11 RF / Wireless (135 total)

**Engineering Relevance:** Wireless connectivity, RF front-end, antenna tuning, coexistence.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| WiFi / WLAN (802.11 a/b/g/n/ac/ax) | 23 | Standard, MIMO, Frequency, Power, Interface |
| Bluetooth / BLE / Mesh / 802.15.4 | 27 | Version, TX Power, Sensitivity, Coexistence, Antenna |
| LoRa / Sub-GHz / Proprietary | 17 | Frequency, Spreading Factor, Bandwidth, Sensitivity |
| Cellular (LTE-M, NB-IoT, 5G NR) | 17 | Bands, Category, Throughput, Power, SIM |
| GNSS (GPS, GLONASS, Galileo, BeiDou, Multi-Constellation) | 13 | Constellations, Sensitivity, TTFF, Accuracy, Antenna |
| RF Front-End (LNA, PA, Switch, Filter, Tuner) | 18 | Frequency, Gain, NF, IP3, Linearity, Control |
| RFID / NFC (HF, UHF, Reader, Tag) | 10 | Standard, Range, Speed, Antenna, Crypto |
| UWB (802.15.4z, Ranging, Radar) | 10 | Band, Precision, Update Rate, Channel |

**Status Distribution:**
- PLANNED: 135

**Golden-Board Relevance:** MEDIUM — Required for wireless golden boards, optional for wired.

---

### 2.12 Audio (85 total)

**Engineering Relevance:** Audio I/O, amplification, DSP, voice, haptics.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Audio ADCs / DACs (I2S, TDM, PDM) | 21 | Resolution, Sample Rate, SNR, THD, Interface |
| Audio Codecs (Integrated ADC/DAC + PLL) | 17 | Channels, Sample Rate, SNR, Features, Interface |
| Class-D Amplifiers (Mono, Stereo, Multi-Channel) | 17 | Power, Efficiency, THD+N, Supply, Protection |
| Class-AB / Headphone Amplifiers | 13 | Power, THD+N, Noise, Gain, Output Config |
| MEMS Microphones (Analog, Digital PDM, I2S) | 8 | SNR, Sensitivity, Frequency Response, Interface |
| Audio DSP / Processors | 9 | MIPS, Memory, Algorithms, Interface |

**Status Distribution:**
- PLANNED: 85

**Golden-Board Relevance:** MEDIUM — Required for audio-enabled golden boards.

---

### 2.13 Protection (135 total)

**Engineering Relevance:** Circuit protection, safety, reliability, compliance — mandatory for production boards.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| TVS / ESD Arrays (Discrete, Multi-Channel) | 31 | Vrwm, Vbr, Ipp (8/20µs), Clamping V, Capacitance |
| Fuses (PTC/Polyfuse, Thermal, Fast/Slow Blow) | 23 | Hold Current, Trip Current, Voltage, Time-Current |
| eFuses / Smart Fuses (Integrated FET + Control) | 18 | Current Limit, Response Time, Auto-Retry, Reporting |
| Overvoltage / Undervoltage Protection | 18 | Threshold, Hysteresis, Response, Output Type |
| Surge Protection (GDT, MOV, Hybrid) | 13 | Surge Rating (8/20, 10/1000), Voltage, Follow Current |
| Reverse Polarity Protection (Ideal Diode, MOSFET) | 13 | Voltage Drop, Current, Reverse Voltage, Thermal |
| Inrush Current Limiters (NTC, Active) | 10 | Resistance, Steady-State Current, Time Constant |
| Thermal Protection (Thermal Fuse, Thermal Switch) | 9 | Trip Temperature, Reset Type, Current Rating |

**Status Distribution:**
- PLANNED: 135

**Golden-Board Relevance:** ESSENTIAL — Every production board needs ESD, overcurrent, overvoltage, reverse polarity protection.

---

### 2.14 Connectors (185 total)

**Engineering Relevance:** Board-to-board, wire-to-board, I/O, power, high-speed — mechanical + electrical interface.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Board-to-Board (Header, Receptacle, Mezzanine, FPC) | 37 | Pitch, Rows, Current, Voltage, Mating Cycles, Stack Height |
| Wire-to-Board (Terminal Block, Crimp Housing, IDC) | 33 | Wire Gauge, Current, Voltage, Pitch, Locking, Tooling |
| USB (Type-A, B, C, Micro, Mini, 3.x) | 18 | Speed, Current, Orientation, Mounting, ESD |
| HDMI / DisplayPort / MIPI DSI/CSI | 14 | Lanes, Speed, Protocol, ESD, Connector Type |
| Ethernet (RJ45, MagJack, M12, Single Pair) | 14 | Speed, PoE, Shielding, Mounting, Isolation |
| PCIe / M.2 / Mini-PCIe / M.3 | 14 | Lanes, Keying, Height, Power, Signal Integrity |
| Automotive (FAKRA, HSD, USCAR, MQS) | 14 | Standard, Coding, Frequency, Vibration, Sealing |
| Power (Barrel, Blade, ATX, PCIe, Custom High-Current) | 18 | Current, Voltage, Pins, Locking, Derating |
| RF / Coaxial (SMA, SMB, MCX, MMCX, U.FL, GSC) | 14 | Frequency, Impedance, VSWR, Power, Mounting |
| Specialized (JTAG/SWD, Camera, Sensor, Industrial) | 9 | Application-specific |

**Status Distribution:**
- PLANNED: 185

**Golden-Board Relevance:** ESSENTIAL — Every board has power, debug, and I/O connectors.

---

### 2.15 Electromechanical (85 total)

**Engineering Relevance:** Human interface, actuation, switching, mechanical integration.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| Switches (Tactile, Slide, DIP, Rotary, Pushbutton) | 26 | Actuation Force, Travel, Cycles, Current/Voltage, Illumination |
| Relays (Signal, Power, Reed, Solid State, Automotive) | 21 | Coil Voltage, Contact Rating, Form, Isolation, Speed |
| Buttons / Keypads / Encoders | 14 | Actuation, Cycles, Resolution, Detent, Interface |
| Fans / Blowers / Thermal Management | 8 | Airflow, Static Pressure, Voltage, PWM, Tach, Noise |
| Motors / Drivers (Stepper, BLDC, Servo, Piezo) | 8 | Torque, Speed, Voltage, Current, Feedback, Driver |
| Vibration / Haptic Actuators (LRA, ERM, Piezo) | 8 | Acceleration, Frequency, Voltage, Drive |

**Status Distribution:**
- PLANNED: 85

**Golden-Board Relevance:** MEDIUM — Required for boards with user interface, motor control, thermal management.

---

### 2.16 Specialized (85 total)

**Engineering Relevance:** Niche applications, emerging technologies, high-reliability, domain-specific.

| Subfamily | Count | Key Engineering Parameters |
|-----------|-------|---------------------------|
| High-Voltage (>1kV) / Isolation Amplifiers | 14 | Working Voltage, Isolation, CMTI, Bandwidth, Accuracy |
| Radiation-Hardened / Space-Grade | 14 | TID, SEE, Dose Rate, Package, Screening Level |
| Automotive AEC-Q100/Q101/Q200 Qualified | 17 | Grade, Temperature, Qualification, PPAP |
| Medical / Implantable / ISO 13485 | 8 | Biocompatibility, Hermeticity, Leakage, Reliability |
| Superconducting / Quantum Computing | 8 | Critical Temperature, Coherence, Junction, Materials |
| Photonic / Silicon Photonics | 8 | Wavelength, Loss, Modulation, Integration |
| Memristor / Neuromorphic / In-Memory Compute | 8 | Conductance States, Endurance, Switching Energy |
| Flexible / Printed / Stretchable Electronics | 8 | Bend Radius, Stretchability, Process Compatibility |

**Status Distribution:**
- PLANNED: 85

**Golden-Board Relevance:** LOW — Specialized demo boards only; not in baseline golden board.

---

## 3. Status Model (Provenance / State)

Every component in the taxonomy carries a **lifecycle status** from this controlled vocabulary:

| Status | Definition | Transition Rules |
|--------|------------|------------------|
| **PLANNED** | Allocated in taxonomy; no datasheet acquired; no assets exist | → VERIFIED (datasheet acquired + validated) |
| **VERIFIED** | Datasheet acquired, parametric facts extracted, validated against source | → AVAILABLE (symbol + footprint + 3D model verified) |
| **AVAILABLE** | Full asset set verified: symbol, footprint (IPC-7351), 3D model (STEP), datasheet hash | → BLOCKED (lifecycle change: NRND/EOL) |
| **BLOCKED** | Part is NRND/EOL, supply constrained, or license/export restricted | → NOT_FOUND (if part disappears entirely) |
| **NOT_FOUND** | Part number invalid, manufacturer unknown, no datasheet locatable | Terminal state (removed from active allocation) |

**State Machine:**
```
PLANNED → VERIFIED → AVAILABLE
                ↓
             BLOCKED → NOT_FOUND
```

**Provenance Tracking:**
Every component record MUST include:
- `source_url`: Authoritative datasheet URL (manufacturer site preferred)
- `source_hash`: SHA-256 of datasheet PDF at acquisition time
- `acquired_at`: ISO 8601 timestamp
- `verified_by`: Agent/run ID that performed verification
- `validation_notes`: Any discrepancies or manual overrides

---

## 4. Family-Specific Metadata Architecture

### 4.1 COMMON CORE (All Families)

Every component, regardless of family, MUST have these fields:

```yaml
common_core:
  # Identity
  component_id: string (ULID)           # Global unique identifier
  mpn: string                           # Manufacturer Part Number
  manufacturer: string                  # Normalized manufacturer name
  family: enum                          # One of 16 families above
  subfamily: string                     # Normalized subfamily name

  # Classification
  component_class: enum                 # Extended ComponentClass enum
  package: string                       # JEDEC/IPC package code (e.g., "0603", "QFN-32")
  pin_count: integer                    # Total pins/pads

  # Lifecycle & Compliance
  lifecycle_status: enum                # active / NRND / EOL
  compliance: object
    rohs: boolean
    reach: boolean
    halogen_free: boolean
    conflict_minerals: boolean
  automotive_qualified: boolean         # AEC-Q100/Q101/Q200 grade if applicable

  # Provenance
  provenance:
    source_url: string
    source_hash: string (SHA-256)
    acquired_at: datetime (ISO 8601)
    verified_by: string
    validation_notes: string?

  # Asset Availability (deterministic flags)
  has_symbol: boolean
  has_footprint: boolean
  has_3d_model: boolean
  footprint_standard: enum              # IPC-7351A/B/C, JEDEC, Custom
  footprint_verified: boolean           # DRC-clean, courtyard validated

  # Engineering Constraints (typed Physical Quantities)
  operating_temperature: PhysicalQuantity  # Range: min/max °C
  max_operating_voltage: PhysicalQuantity? # If applicable
  max_power_dissipation: PhysicalQuantity? # If applicable

  # Search & Discovery
  tags: string[]                        # Searchable keywords
  description: string                   # One-line functional description
  datasheet_url: string?                # Current manufacturer URL
```

### 4.2 FAMILY-SPECIFIC Metadata

Each family adds required fields. **All quantities use the Physical Quantity type system (magnitude + unit + tolerance).**

---

#### 4.2.1 Passives Family-Specific

```yaml
passives_specific:
  # Resistors
  resistance?: PhysicalQuantity         # Nominal ± tolerance
  power_rating?: PhysicalQuantity       # Watts @ 70°C
  max_working_voltage?: PhysicalQuantity
  temperature_coefficient?: PhysicalQuantity  # ppm/°C
  voltage_coefficient?: PhysicalQuantity    # ppm/V
  noise_index?: PhysicalQuantity        # µV/V (decade)
  pulse_withstand?: PhysicalQuantity    # Energy or peak power

  # Capacitors
  capacitance?: PhysicalQuantity        # Nominal ± tolerance
  voltage_rating?: PhysicalQuantity     # DC voltage
  esr?: PhysicalQuantity                # Equivalent Series Resistance @ freq
  esl?: PhysicalQuantity                # Equivalent Series Inductance
  dielectric_type?: enum                # X7R, X5R, C0G, Y5V, Aluminum, Tantalum, Film
  dc_bias_derating_curve?: reference    # Link to derating data
  ripple_current_rating?: PhysicalQuantity # RMS @ frequency/temp

  # Inductors
  inductance?: PhysicalQuantity         # Nominal ± tolerance @ test freq
  dcr?: PhysicalQuantity                # DC Resistance
  saturation_current?: PhysicalQuantity # Isat @ % drop (typically 20-30%)
  rms_current?: PhysicalQuantity        # Irms @ ΔT (typically 40°C rise)
  srf?: PhysicalQuantity                # Self-Resonant Frequency
  q_factor?: PhysicalQuantity           # @ test frequency
  core_material?: enum                  # Ferrite, Powdered Iron, Air, etc.

  # Crystals/Oscillators
  frequency?: PhysicalQuantity
  load_capacitance?: PhysicalQuantity
  esr?: PhysicalQuantity                # Equivalent Series Resistance
  drive_level?: PhysicalQuantity        # Max drive power
  frequency_stability?: PhysicalQuantity # ppm over temp
  aging?: PhysicalQuantity              # ppm/year
```

---

#### 4.2.2 Diodes Family-Specific

```yaml
diodes_specific:
  forward_voltage?: PhysicalQuantity    # Vf @ If
  reverse_voltage?: PhysicalQuantity    # Vrrm / Vr
  forward_current?: PhysicalQuantity    # If (avg) / Ifsm (surge)
  reverse_recovery_time?: PhysicalQuantity # Trr
  junction_capacitance?: PhysicalQuantity # Cj @ Vr
  leakage_current?: PhysicalQuantity    # Ir @ Vr
  power_dissipation?: PhysicalQuantity
  thermal_resistance_jc?: PhysicalQuantity # Junction-to-case
  thermal_resistance_ja?: PhysicalQuantity # Junction-to-ambient
  
  # Zener-specific
  zener_voltage?: PhysicalQuantity      # Vz @ Izt
  zener_impedance?: PhysicalQuantity    # Zzt @ Izt
  temperature_coefficient?: PhysicalQuantity # mV/°C or %/°C
  
  # TVS-specific
  reverse_working_voltage?: PhysicalQuantity # Vrwm
  breakdown_voltage?: PhysicalQuantity  # Vbr @ Ibr
  peak_pulse_current?: PhysicalQuantity # Ipp @ 8/20µs or 10/1000µs
  clamping_voltage?: PhysicalQuantity   # Vc @ Ipp
  
  # LED-specific
  luminous_intensity?: PhysicalQuantity # mcd @ If
  dominant_wavelength?: PhysicalQuantity # nm
  viewing_angle?: PhysicalQuantity      # degrees
  color_temperature?: PhysicalQuantity  # K (white LEDs)
```

---

#### 4.2.3 Transistors Family-Specific

```yaml
transistors_specific:
  # MOSFET
  drain_source_voltage?: PhysicalQuantity # Vds
  gate_source_voltage?: PhysicalQuantity  # Vgs max
  continuous_drain_current?: PhysicalQuantity # Id @ Tc
  pulsed_drain_current?: PhysicalQuantity   # Idm
  rds_on?: PhysicalQuantity               # @ Vgs, Id
  gate_threshold_voltage?: PhysicalQuantity # Vgs(th)
  total_gate_charge?: PhysicalQuantity      # Qg
  input_capacitance?: PhysicalQuantity      # Ciss
  output_capacitance?: PhysicalQuantity     # Coss
  reverse_transfer_capacitance?: PhysicalQuantity # Crss
  turn_on_delay?: PhysicalQuantity
  rise_time?: PhysicalQuantity
  turn_off_delay?: PhysicalQuantity
  fall_time?: PhysicalQuantity
  
  # BJT
  collector_emitter_voltage?: PhysicalQuantity # Vceo
  collector_current?: PhysicalQuantity         # Ic
  dc_current_gain?: PhysicalQuantity           # hFE @ Ic, Vce
  collector_emitter_sat_voltage?: PhysicalQuantity # Vce(sat)
  transition_frequency?: PhysicalQuantity      # ft
  power_dissipation?: PhysicalQuantity
  
  # IGBT/GaN/SiC
  collector_emitter_voltage?: PhysicalQuantity # Vces
  gate_emitter_threshold?: PhysicalQuantity    # Vge(th)
  collector_emitter_sat?: PhysicalQuantity     # Vce(sat)
  turn_on_energy?: PhysicalQuantity            # Eon
  turn_off_energy?: PhysicalQuantity           # Eoff
  reverse_recovery_charge?: PhysicalQuantity   # Qrr
```

---

#### 4.2.4 Analog ICs / Op-Amps Family-Specific

```yaml
analog_ics_specific:
  # Op-Amps
  gain_bandwidth_product?: PhysicalQuantity # GBW
  slew_rate?: PhysicalQuantity              # V/µs
  input_offset_voltage?: PhysicalQuantity   # Vos
  input_offset_voltage_drift?: PhysicalQuantity # µV/°C
  input_bias_current?: PhysicalQuantity     # Ib
  input_offset_current?: PhysicalQuantity   # Ios
  input_voltage_noise?: PhysicalQuantity    # nV/√Hz @ 1kHz
  input_current_noise?: PhysicalQuantity    # pA/√Hz
  cmrr?: PhysicalQuantity                   # dB
  psrr?: PhysicalQuantity                   # dB
  supply_voltage_range?: PhysicalQuantity   # Min/Max
  quiescent_current?: PhysicalQuantity      # Iq per amplifier
  output_current?: PhysicalQuantity         # Isc / Io
  rail_to_rail_input?: boolean
  rail_to_rail_output?: boolean
  
  # Comparators
  propagation_delay?: PhysicalQuantity      # High-to-Low, Low-to-High
  overdrive_voltage?: PhysicalQuantity
  hysteresis?: PhysicalQuantity
  output_type?: enum                        # Open-drain, Push-pull
  
  # Voltage References
  output_voltage?: PhysicalQuantity
  initial_accuracy?: PhysicalQuantity       # %
  temperature_coefficient?: PhysicalQuantity # ppm/°C
  noise_0_1_10hz?: PhysicalQuantity         # µVpp
  noise_10hz_10khz?: PhysicalQuantity       # µVrms
  load_regulation?: PhysicalQuantity        # ppm/mA
  line_regulation?: PhysicalQuantity        # ppm/V
  
  # ADC/DAC
  resolution?: integer                      # bits
  sample_rate?: PhysicalQuantity            # SPS
  integral_nonlinearity?: PhysicalQuantity  # LSB
  differential_nonlinearity?: PhysicalQuantity # LSB
  snr?: PhysicalQuantity                    # dB
  thd?: PhysicalQuantity                    # dB
  interface?: enum                          # SPI, I2C, Parallel
  reference_type?: enum                     # Internal, External, Both
```

---

#### 4.2.5 Power Management Family-Specific

```yaml
power_mgmt_specific:
  # DC-DC Converters
  input_voltage_range?: PhysicalQuantity    # Vin min/max
  output_voltage_range?: PhysicalQuantity   # Vout min/max (fixed or adj)
  output_current?: PhysicalQuantity         # Max Iout
  switching_frequency?: PhysicalQuantity    # Fixed or range
  efficiency?: PhysicalQuantity             # % @ typical conditions
  control_mode?: enum                       # PWM, PFM, PSM, Current Mode, Voltage Mode
  enable_pin?: boolean
  sync_pin?: boolean
  power_good?: boolean
  soft_start?: PhysicalQuantity             # Adjustable/fixed time
  
  # LDO
  dropout_voltage?: PhysicalQuantity        # @ Iout
  line_regulation?: PhysicalQuantity        # %/V
  load_regulation?: PhysicalQuantity        # %/A
  output_noise?: PhysicalQuantity           # µVrms (10Hz-100kHz)
  psrr?: PhysicalQuantity                   # dB @ frequency
  quiescent_current?: PhysicalQuantity      # Iq (no load)
  shutdown_current?: PhysicalQuantity       # Isd
  current_limit?: PhysicalQuantity
  thermal_shutdown?: PhysicalQuantity       # Temperature
  
  # Battery Management
  battery_chemistry?: enum                  # Li-Ion, LiFePO4, NiMH, Lead Acid
  charge_voltage?: PhysicalQuantity
  charge_current?: PhysicalQuantity
  termination_current?: PhysicalQuantity
  precharge_current?: PhysicalQuantity
  safety_timer?: PhysicalQuantity
  fuel_gauge_algorithm?: enum               # Coulomb counting, impedance tracking
  cell_count?: integer
  balancing_current?: PhysicalQuantity
  
  # Protection
  current_limit_accuracy?: PhysicalQuantity
  response_time?: PhysicalQuantity          # Overcurrent, short-circuit
  auto_retry?: boolean
  fault_reporting?: enum                    # Pin, I2C, Both
```

---

#### 4.2.6 Digital Logic Family-Specific

```yaml
digital_logic_specific:
  propagation_delay?: PhysicalQuantity      # tpd
  output_drive_current?: PhysicalQuantity   # Ioh/Iol
  input_voltage_high?: PhysicalQuantity     # Vih
  input_voltage_low?: PhysicalQuantity      # Vil
  output_voltage_high?: PhysicalQuantity    # Voh @ Ioh
  output_voltage_low?: PhysicalQuantity     # Vol @ Iol
  supply_voltage_range?: PhysicalQuantity
  quiescent_current?: PhysicalQuantity
  input_capacitance?: PhysicalQuantity
  max_frequency?: PhysicalQuantity          # Toggle frequency
  
  # Level Translators
  voltage_range_a?: PhysicalQuantity        # Vcca
  voltage_range_b?: PhysicalQuantity        # Vccb
  direction_control?: enum                  # Auto, DIR pin, Output enable
  
  # Clock/Buffers
  additive_jitter?: PhysicalQuantity        # fs RMS
  output_skew?: PhysicalQuantity
  duty_cycle_distortion?: PhysicalQuantity
  output_format?: enum                      # LVCMOS, LVDS, LVPECL, HCSL, CML
```

---

#### 4.2.7 MCUs Family-Specific

```yaml
mcus_specific:
  core_architecture?: enum                  # Cortex-M0/3/4/7/33/35P, RISC-V, AVR, etc.
  core_frequency?: PhysicalQuantity         # Max CPU frequency
  flash_size?: PhysicalQuantity             # KB/MB
  ram_size?: PhysicalQuantity               # KB
  eeprom_size?: PhysicalQuantity?           # Bytes
  gpio_count?: integer
  adc_channels?: integer
  adc_resolution?: integer                  # bits
  dac_channels?: integer?
  dac_resolution?: integer?
  timers?: object
    count_16bit?: integer
    count_32bit?: integer
    advanced_control?: integer
    basic?: integer
    lptim?: integer
  communication_peripherals?: object
    uart?: integer
    spi?: integer
    i2c?: integer
    i3c?: integer
    can?: integer
    can_fd?: boolean
    usb?: enum                              # None, FS, HS, OTG, PD
    ethernet?: boolean
    sdio?: boolean
  security_features?: object
    trustzone?: boolean
    mpu?: boolean
    secure_boot?: boolean
    hardware_crypto?: string[]              # AES, SHA, RSA, ECC, TRNG
    tamper_detection?: boolean
  package_options?: string[]                # LQFP, BGA, WLCSP, QFN
  temperature_grade?: enum                  # Industrial, Extended, Automotive
```

---

#### 4.2.8 Memory Family-Specific

```yaml
memory_specific:
  density?: PhysicalQuantity                # Bits/Bytes (Kb, Mb, Gb)
  organization?: string                     # e.g., "1M x 8", "256K x 16"
  interface?: enum                          # SPI, QSPI, OSPI, I2C, Parallel, SD, eMMC
  max_frequency?: PhysicalQuantity          # Clock frequency
  supply_voltage?: PhysicalQuantity         # Vcc range
  read_current?: PhysicalQuantity
  write_current?: PhysicalQuantity
  standby_current?: PhysicalQuantity
  page_size?: integer                       # Bytes
  sector_size?: integer                     # Bytes
  block_size?: integer                      # Bytes
  erase_time?: PhysicalQuantity             # Sector/Block/Chip
  write_time?: PhysicalQuantity             # Page/Byte
  endurance_cycles?: integer                # Program/Erase cycles
  data_retention?: PhysicalQuantity         # Years @ temperature
  hardware_ecc?: boolean
  write_protect?: enum                      # Hardware, Software, Both
  unique_id?: boolean
  security_features?: string[]              # OTP, Lockable regions, Encryption
```

---

#### 4.2.9 Communication Family-Specific

```yaml
communication_specific:
  protocol?: enum                           # UART, SPI, I2C, USB, Ethernet, CAN, etc.
  max_data_rate?: PhysicalQuantity          # bps, Mbps, Gbps
  voltage_levels?: PhysicalQuantity         # I/O voltage range
  channels?: integer                        # Number of ports/channels
  isolation_voltage?: PhysicalQuantity?     # If isolated
  cmti?: PhysicalQuantity?                  # Common Mode Transient Immunity (kV/µs)
  esd_protection?: PhysicalQuantity         # HBM/CDM rating
  transceiver_integrated?: boolean
  phy_integrated?: boolean
  mac_integrated?: boolean
  buffer_size?: PhysicalQuantity            # FIFO/Buffer depth
  dma_support?: boolean
  wakeup_capability?: boolean
  automotive_qualified?: boolean
```

---

#### 4.2.10 Sensors Family-Specific

```yaml
sensors_specific:
  sensor_type?: enum                        # Temperature, Pressure, Accel, Gyro, Mag, Optical, Current, etc.
  measurement_range?: PhysicalQuantity      # Min/Max
  accuracy?: PhysicalQuantity               # ±% FS or absolute
  resolution?: PhysicalQuantity             # LSB size
  sensitivity?: PhysicalQuantity            # Output per unit (e.g., mV/g, LSB/°C)
  noise_density?: PhysicalQuantity          # µg/√Hz, µT/√Hz, etc.
  bandwidth?: PhysicalQuantity              # Hz
  output_interface?: enum                   # I2C, SPI, Analog, PWM, 1-Wire, SMBus
  supply_voltage?: PhysicalQuantity
  current_consumption?: PhysicalQuantity    # Active, Sleep, Shutdown
  temperature_range?: PhysicalQuantity
  response_time?: PhysicalQuantity
  calibration?: enum                        # Factory, User, None
  fiducial_reference?: boolean              # For optical/ToF
```

---

#### 4.2.11 RF / Wireless Family-Specific

```yaml
rf_wireless_specific:
  frequency_range?: PhysicalQuantity        # Min/Max frequency
  tx_power?: PhysicalQuantity               # dBm (max, typical)
  rx_sensitivity?: PhysicalQuantity         # dBm @ BER/PER
  modulation_schemes?: string[]             # OFDM, DSSS, FSK, LoRa, etc.
  channel_bandwidth?: PhysicalQuantity
  antenna_interface?: enum                  # Single-ended, Differential, Balun integrated
  pa_integrated?: boolean
  lna_integrated?: boolean
  coexistence_support?: boolean
  regulatory_certifications?: string[]      # FCC, CE, IC, MIC, SRRC
  current_tx?: PhysicalQuantity             # mA @ max power
  current_rx?: PhysicalQuantity             # mA
  current_sleep?: PhysicalQuantity          # µA
  link_budget?: PhysicalQuantity            # dB
```

---

#### 4.2.12 Audio Family-Specific

```yaml
audio_specific:
  resolution?: integer                      # bits (16, 24, 32)
  sample_rates?: PhysicalQuantity[]         # Supported rates (8k-768k)
  snr?: PhysicalQuantity                    # dB (A-weighted)
  thd_n?: PhysicalQuantity                  # % or dB
  dynamic_range?: PhysicalQuantity          # dB
  channel_count?: integer
  interface?: enum                          # I2S, TDM, PDM, SPI, I2C
  master_clock?: PhysicalQuantity           # MCLK requirement
  pll_integrated?: boolean
  headphone_amp_power?: PhysicalQuantity    # mW @ load
  speaker_amp_power?: PhysicalQuantity      # W @ load, THD
  mic_bias?: PhysicalQuantity               # V, mA
  dsp_features?: string[]                   # EQ, DRC, ANC, Beamforming
```

---

#### 4.2.13 Protection Family-Specific

```yaml
protection_specific:
  protection_type?: enum                    # TVS, Fuse, eFuse, OVP, UVP, RPP, Surge, Thermal
  working_voltage?: PhysicalQuantity        # Vrwm / Vdc max
  breakdown_voltage?: PhysicalQuantity      # Vbr
  clamping_voltage?: PhysicalQuantity       # Vc @ Ipp
  peak_pulse_current?: PhysicalQuantity     # Ipp @ waveform
  peak_pulse_power?: PhysicalQuantity       # Ppp @ waveform
  hold_current?: PhysicalQuantity           # PTC/Ihold
  trip_current?: PhysicalQuantity           # PTC/Itrip
  trip_time?: PhysicalQuantity              # @ overcurrent multiple
  reset_type?: enum                         # Auto, Power-cycle, Manual
  response_time?: PhysicalQuantity          # ns/µs/ms
  leakage_current?: PhysicalQuantity        # @ working voltage
  capacitance?: PhysicalQuantity            # Junction/parasitic
  isolation_voltage?: PhysicalQuantity?     # If isolated protection
  bidirectional?: boolean                   # For TVS arrays
```

---

#### 4.2.14 Connectors Family-Specific

```yaml
connectors_specific:
  connector_type?: enum                     # B2B, W2B, USB, HDMI, Ethernet, Power, RF, etc.
  pitch?: PhysicalQuantity                  # mm
  positions?: integer                       # Number of contacts
  rows?: integer                            # 1, 2, 3, 4...
  current_per_contact?: PhysicalQuantity    # A
  voltage_rating?: PhysicalQuantity         # V
  mating_cycles?: integer
  mounting_style?: enum                     # Through-hole, SMT, Press-fit, Edge
  orientation?: enum                        # Vertical, Right-angle, Horizontal
  locking_mechanism?: enum                  # None, Latch, Screw, Lever, Friction
  shielding?: boolean
  impedance_controlled?: boolean            # For high-speed
  differential_pairs?: integer              # Count
  operating_temperature?: PhysicalQuantity
  flange_mount?: boolean
  panel_mount?: boolean
  cable_accommodation?: string              # Wire gauge range, cable type
```

---

#### 4.2.15 Electromechanical Family-Specific

```yaml
electromechanical_specific:
  device_type?: enum                        # Switch, Relay, Button, Encoder, Fan, Motor, Actuator
  actuation_force?: PhysicalQuantity        # N or gf
  actuation_travel?: PhysicalQuantity       # mm
  mechanical_life?: integer                 # Cycles
  electrical_life?: integer                 # Cycles @ rated load
  contact_rating?: PhysicalQuantity         # V, A (resistive/inductive)
  coil_voltage?: PhysicalQuantity?          # For relays
  coil_power?: PhysicalQuantity?            # For relays
  contact_form?: enum?                      # SPST, SPDT, DPDT, etc. (relays)
  resolution?: PhysicalQuantity?            # PPR, CPR (encoders)
  detent?: boolean?                         # Encoders/switches
  airflow?: PhysicalQuantity?               # CFM (fans)
  static_pressure?: PhysicalQuantity?       # mmH2O (fans)
  torque?: PhysicalQuantity?                # Nm (motors)
  speed?: PhysicalQuantity?                 # RPM (motors)
  feedback_type?: enum?                     # None, Hall, Encoder, Resolver (motors)
```

---

#### 4.2.16 Specialized Family-Specific

```yaml
specialized_specific:
  application_domain?: enum                 # High-Voltage, Space, Automotive, Medical, Quantum, Photonic, Neuromorphic, Flexible
  qualification_standard?: string           # AEC-Q100, MIL-PRF-38535, ECSS-Q-ST-60, ISO 13485, etc.
  radiation_tolerance?: PhysicalQuantity?   # krad(Si), SEE LET threshold
  operating_temperature_extreme?: PhysicalQuantity # Extended range
  hermetic_sealing?: boolean?
  biocompatibility?: enum?                  # ISO 10993 class
  vacuum_compatible?: boolean?
  magnetic_field_immunity?: PhysicalQuantity?
  special_requirements?: string[]           # Free-form specialized constraints
```

---

## 5. Engineering Constraints Per Family

| Family | Critical Constraints | DRC/ERC Rules | Thermal Considerations |
|--------|---------------------|---------------|------------------------|
| **Passives** | Power derating (≤50% for resistors, ≤80% for caps voltage), DC bias derating (MLCC), current rating (inductors), self-heating | Symbol pin count = footprint pad count; polarity marking for electrolytic/tantalum; courtyard clearance | Resistor: θ_JA from datasheet; Capacitor: ripple current heating; Inductor: core + copper loss |
| **Diodes** | Thermal: Pd = Vf × If; reverse voltage derating (≥20% margin); surge capability | Polarity marking; cathode indicator on footprint; thermal vias for power diodes | Junction-to-ambient θ_JA; heatsink required >500mW; thermal relief on pads |
| **Transistors** | SOA (Safe Operating Area); Rds(on) at Vgs; gate drive requirements; switching losses | Gate resistor footprint; thermal pad connection; Kelvin source for high-current | θ_JC critical; PCB copper as heatsink; via stitching under thermal pad |
| **Analog ICs** | Supply sequencing; input common-mode range; output load capability; noise budget | Analog ground separation; guard rings; Kelvin connections for precision | Low power typically; thermal shutdown protection |
| **Power Mgmt** | Input/output capacitor specs (ESR, value); inductor selection; thermal design; loop stability | Power stage layout: input cap → IC → inductor → output cap; thermal vias; phase separation | θ_JA from datasheet; multilayer thermal design; efficiency → heat |
| **Digital Logic** | Vih/Vil margins; signal integrity (overshoot, ringing); simultaneous switching noise (SSO) | Decoupling per power pin; controlled impedance for >50MHz; length matching for buses | Low power; thermal mainly from output loading |
| **MCUs** | Power sequencing (core, I/O, analog); clock stability; reset timing; debug access | Decoupling per VDD pin; crystal layout (guard ring, short traces); SWD/JTAG access | Dynamic power ∝ f × V²; thermal management for >100MHz; LPM modes |
| **Memory** | Timing parameters (setup, hold, access); signal integrity; power-up sequencing | Length matching for parallel; controlled impedance for high-speed serial; decoupling | Low power typically; DDR requires thermal consideration |
| **Communication** | Signal integrity (eye diagram); impedance control (90Ω diff, 50Ω SE); isolation barriers | Differential pair routing; length matching; common-mode chokes; isolation creepage | PHY power can be significant; thermal pad for high-speed |
| **Sensors** | Noise floor; cross-axis sensitivity; mounting stress; calibration; interface timing | Analog sensor: guard traces, star ground; digital: pull-ups, bus capacitance | Self-heating affects accuracy; thermal isolation from PCB |
| **RF/Wireless** | 50Ω impedance control; ground vias stitching; antenna keepout; harmonics; coexistence | Coplanar waveguide; via fences; antenna matching network placement; FCC/CE compliance | PA efficiency → heat; thermal via under RFIC; LNA sensitivity vs temperature |
| **Audio** | PSRR; ground loop prevention; EMI filtering; pop/click suppression | Analog/digital ground separation; star ground; filter placement near connector | Class-D: inductor thermal; speaker amp: heatsink |
| **Protection** | Clamping voltage < protected device max; response time < threat duration; power dissipation during event | Placement at connector entry; minimal trace inductance to ground; thermal relief | Energy absorption → transient heating; thermal mass consideration |
| **Connectors** | Current derating per contact; insertion force; vibration/shock; mating cycles | Courtyard for mating envelope; keepout for ejector/latch; high-speed: impedance, length matching | Power contacts: current density, temperature rise |
| **Electromechanical** | Actuation force/travel; mechanical life; vibration resistance; mounting stress | Keepout for actuator movement; panel cutout alignment; strain relief | Motor drivers: thermal; relays: coil heating |
| **Specialized** | Domain-specific (radiation, vacuum, biocompatibility, cryogenic) | Domain-specific | Domain-specific |

---

## 6. Golden-Board Relevance Per Family

| Family | Relevance | Role in Golden Board | Minimum Components Required |
|--------|-----------|---------------------|----------------------------|
| **Passives** | **ESSENTIAL** | Power integrity (decoupling, bulk), signal integrity (termination, filtering), timing (crystals), EMI (ferrites) | 100+ |
| **Diodes** | **ESSENTIAL** | Reverse polarity protection, ESD clamping (TVS), power rectification, status indication (LED) | 15+ |
| **Transistors** | **HIGH** | Load switches, level translation, motor/relay drive, power path control | 10+ |
| **Analog ICs** | **HIGH** | Sensor signal conditioning, power monitoring (INA), voltage reference, audio | 10+ |
| **Power Mgmt** | **ESSENTIAL** | All voltage rails (buck, boost, LDO), sequencing, battery management, protection | 15+ |
| **Digital Logic** | **HIGH** | Level translation, clock buffering, bus buffering, glue logic | 10+ |
| **MCUs** | **ESSENTIAL** | Main application processor, wireless co-processor, real-time control | 2-3 |
| **Memory** | **HIGH** | Boot flash (SPI NOR), config EEPROM, data logging (FRAM), external RAM | 5+ |
| **Communication** | **ESSENTIAL** | USB (debug/power), UART (console), I2C/SPI (peripherals), Ethernet/CAN (field), isolation | 10+ |
| **Sensors** | **HIGH** | Temperature, current/voltage/power monitoring, IMU (if motion), environmental | 5+ |
| **RF/Wireless** | **MEDIUM** | BLE/WiFi/LoRa module or chipset + front-end (if wireless golden board) | 3-5 |
| **Audio** | **MEDIUM** | Codec + amp (if audio golden board) | 2-3 |
| **Protection** | **ESSENTIAL** | ESD at every connector, input fuse/eFuse, reverse polarity, surge (if outdoor) | 10+ |
| **Connectors** | **ESSENTIAL** | Power input, debug (JTAG/SWD), USB, I/O headers, board-to-board | 8+ |
| **Electromechanical** | **MEDIUM** | Reset button, user button, DIP switch (config), fan header (if thermal) | 4+ |
| **Specialized** | **LOW** | Only for specialized demo variants (automotive, space, medical) | 0-2 |

**Golden-Board Minimum Component Count: ~220 unique components**

---

## 7. Verification Requirements Per Family

| Family | Datasheet Verification | Symbol Verification | Footprint Verification | 3D Model | Electrical Validation |
|--------|----------------------|-------------------|----------------------|----------|---------------------|
| **Passives** | Parametric extraction (R/C/L, ratings, derating curves) | IEEE/IEC standard symbols; polarity for polarized | IPC-7351 land patterns; courtyard per IPC; polarity mark | STEP for inductors >0805; optional for 0603/0402 | SPICE model validation; derating curves |
| **Diodes** | Vf/If curves; Vrrm; Trr; capacitance vs voltage | Standard diode symbol; Zener/TVS/LED variants | IPC-7351; cathode mark; thermal pad for power | TO-252/DPAK/D2PAK: STEP with thermal pad | SPICE model (if available); thermal derating |
| **Transistors** | Transfer curves; output characteristics; switching; SOA | MOSFET/BJT/IGBT/GaN symbols; pin mapping | IPC-7351; thermal pad land pattern; Kelvin source | Power packages: STEP with thermal pad | SPICE model; switching loss calculation |
| **Analog ICs** | All datasheet tables; typical curves; noise; distortion | Functional block symbol; pin names per datasheet | IPC-7351; exposed pad if present | QFN/TQFP/BGA: STEP | SPICE macromodel; noise simulation |
| **Power Mgmt** | Efficiency curves; control loop; protection thresholds; layout guidelines | Functional symbol with pin groups (power, control, feedback) | IPC-7351; thermal pad critical; input/output cap placement | QFN/TSSOP/BGA: STEP | SIMPLIS/SPICE avg model; loop stability |
| **Digital Logic** | Timing params; drive strength; voltage translation | IEEE Std 91/91a logic symbols | IPC-7351 standard | Optional (SOIC/TSSOP/QFN) | IBIS model for signal integrity |
| **MCUs** | Electrical characteristics; peripheral specs; errata | Functional symbol (peripheral groups); power/ground clusters | BGA/LQFP/QFN: IPC-7351; fanout strategy | BGA/LQFP: STEP | IBIS; power estimation spreadsheet |
| **Memory** | Timing diagrams; command sequences; endurance | Functional symbol; address/data/control groups | BGA/TSOP/SON: IPC-7351; controlled impedance | BGA: STEP | IBIS; signal integrity simulation |
| **Communication** | Protocol compliance; timing; voltage levels; isolation specs | Functional symbol; differential pairs marked | IPC-7351; controlled impedance pads; isolation creepage | Optional | IBIS; eye diagram; CMTI verification |
| **Sensors** | Transfer function; noise; cross-sensitivity; calibration | Functional symbol; pin functions | IPC-7351; mechanical keepout for sensing element | STEP with sensing element orientation | Noise analysis; calibration procedure |
| **RF/Wireless** | S-parameters; NF; IP3; harmonics; regulatory | Functional symbol; RF pins differential | 50Ω controlled impedance; ground via pattern; antenna keepout | Module: STEP with antenna | S-parameter validation; link budget |
| **Audio** | FFT (THD+N, SNR); frequency response; crosstalk | Functional symbol; analog/digital sections | IPC-7351; analog ground isolation | Optional | Audio precision analyzer correlation |
| **Protection** | I-V curves; clamping vs current; derating; aging | IEEE symbol; bidirectional marking | IPC-7351; minimal inductance to ground | Optional | TLP/IEC 61000-4-2 validation |
| **Connectors** | Mechanical specs; current derating; mating cycles | JEDEC/IEC symbol; pin numbering | Manufacturer land pattern; mating envelope keepout | STEP (critical for mechanical) | Signal integrity (high-speed); thermal |
| **Electromechanical** | Force/displacement; life cycles; electrical rating | Functional symbol | Manufacturer pattern; actuator keepout | STEP (critical) | Contact resistance; bounce time |
| **Specialized** | Domain-specific qualification data | Domain-specific | Domain-specific | Domain-specific | Domain-specific |

---

## 8. Asset Requirements Per Family

| Family | Symbol | Footprint (IPC-7351) | 3D Model (STEP) | SPICE/IBIS | Datasheet |
|--------|--------|---------------------|-----------------|------------|-----------|
| Passives | ✓ Required | ✓ Required | Inductors ≥0805 | SPICE (R/L/C models) | ✓ Required |
| Diodes | ✓ Required | ✓ Required | Power packages | SPICE (if avail) | ✓ Required |
| Transistors | ✓ Required | ✓ Required | Power packages | SPICE (required) | ✓ Required |
| Analog ICs | ✓ Required | ✓ Required | All packages | SPICE macromodel | ✓ Required |
| Power Mgmt | ✓ Required | ✓ Required | All packages | SIMPLIS avg model | ✓ Required |
| Digital Logic | ✓ Required | ✓ Required | Optional | IBIS (high-speed) | ✓ Required |
| MCUs | ✓ Required | ✓ Required | BGA/LQFP/QFN | IBIS | ✓ Required |
| Memory | ✓ Required | ✓ Required | BGA | IBIS | ✓ Required |
| Communication | ✓ Required | ✓ Required | Optional | IBIS | ✓ Required |
| Sensors | ✓ Required | ✓ Required | With sensor element | Optional | ✓ Required |
| RF/Wireless | ✓ Required | ✓ Required (50Ω) | Module | S-parameters | ✓ Required |
| Audio | ✓ Required | ✓ Required | Optional | Optional | ✓ Required |
| Protection | ✓ Required | ✓ Required | Optional | Optional | ✓ Required |
| Connectors | ✓ Required | ✓ Required | ✓ Required | IBIS (high-speed) | ✓ Required |
| Electromechanical | ✓ Required | ✓ Required | ✓ Required | Optional | ✓ Required |
| Specialized | ✓ Required | ✓ Required | Domain-specific | Domain-specific | ✓ Required |

**Asset Verification Gates:**
1. **SYMBOL_GATE**: Pin count matches footprint; pin names match datasheet; graphical standard compliance
2. **FOOTPRINT_GATE**: IPC-7351 compliance; courtyard clearance ≥0.25mm; pad dimensions ±0.05mm; thermal pad via pattern; polarity mark
3. **3D_GATE**: STEP AP214/AP242; correct origin (center/seating plane); body dimensions ±0.1mm; lead/tip representation; no intersecting solids
4. **SIMULATION_GATE**: SPICE/IBIS/S-param model converges; matches datasheet typical curves; passes corner cases

---

## 9. Implementation Phases & Dependencies

### Phase 1: Foundation (Week 1-2)
- [x] TASK-001: Create authoritative taxonomy (this document)
- [ ] TASK-002: Validate 3,500 allocation arithmetic
- [ ] TASK-003: Define family-specific metadata architecture (Sections 4.1-4.2)

### Phase 2: Golden Board Mapping (Week 2-3)
- [ ] TASK-004: Create `docs/engineering/eak-v1-demo-component-matrix.md`
  - Map 3,500 → golden board BOM
  - Essential vs supporting families
  - Engineering capability demonstrated per family
  - Asset requirements per family
  - Verification dependencies
  - Current availability status
  - Blockers & prerequisites

### Phase 3: Validation Automation (Week 3)
- [ ] TASK-005: Create deterministic validation script (`scripts/validate_taxonomy.py`)
  - Parse taxonomy markdown/YAML
  - Verify total = 3,500
  - Check no duplicate MPNs
  - Validate family/subfamily enum values
  - Check required fields per family
  - Verify current + additional = total per family
  - Output machine-readable JSON + human report

### Phase 4: Day 2 Report (Week 3-4)
- [ ] TASK-006: Generate `reports/day2-taxonomy-60min.md`
  - 3,500 allocation summary
  - Family breakdown table
  - Metadata architecture summary
  - Files created/modified
  - Validation test results
  - Failures and fixes
  - Blockers
  - Golden-board matrix status
  - Exact next engineering task

---

## 10. Next Immediate Actions

| Priority | Task | Owner | Depends On |
|----------|------|-------|------------|
| **P0** | Run validation script on this taxonomy | Terminal 1 | TASK-001 complete |
| **P0** | Create `docs/engineering/eak-v1-demo-component-matrix.md` | Terminal 1 | TASK-001, TASK-003 |
| **P0** | Implement `scripts/validate_taxonomy.py` | Terminal 1 | TASK-001 |
| **P1** | Extend `ComponentClass` enum in code for all 16 families | Terminal 2 (coordination) | TASK-003 |
| **P1** | Design family-specific metadata structs in Rust | Terminal 2 (coordination) | TASK-003 |
| **P1** | Plan datasheet acquisition priority (high-relevance families first) | Terminal 1 | Golden-board matrix |

---

## 11. Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-08-20 | Terminal 1 | Initial authoritative taxonomy creation |

---

## 12. Validation Checklist (Pre-Commit)

- [ ] Total components = 3,500
- [ ] Current verified = 500 (Passives only)
- [ ] Additional planned = 3,000
- [ ] 16 families represented
- [ ] All subfamily counts sum to family totals
- [ ] All family totals sum to 3,500
- [ ] No negative counts
- [ ] No duplicate family names
- [ ] Status model defined (5 states)
- [ ] COMMON CORE fields defined
- [ ] FAMILY-SPECIFIC fields defined for all 16 families
- [ ] Engineering constraints documented per family
- [ ] Golden-board relevance scored per family
- [ ] Verification requirements matrix complete
- [ ] Asset requirements matrix complete
- [ ] Implementation phases defined
- [ ] Next actions prioritized

---

**END OF TAXONOMY DOCUMENT**

*This document is the single source of truth for EAK V1 component library scope. All downstream work (data model, acquisition pipeline, asset generation, search, agents) derives from this taxonomy.*