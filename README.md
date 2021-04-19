# Realtime OFDM with CUDA Acceleration

This repository holds the code and whitepaper for my final Principles of Wireless project at Olin College of Engineering. This project implements a full, realtime, OFDM datalink between an Ettus B210 SDR receiver and transmitter.

The final algorithms and code presented in this repository implement a communication system capable of XXX gb/s communication with an error rate of xxxx. The final channel features these techniques:

- 64QAM
- Frequency Correction with Shmidl-Cox algorithm
- Phase Offset Correction
- Guardbands 
- Error Correction with Hamming Codes

The "video" demo features an application using gstreamer to send and receive data through the channel, as a proof of concept of live video streaming. 

To make it easier for fellow and future classmates, I've taken the liberty of bundling .exe/.deb/.app for Windows/Linux/Mac. This should make it easier to send and receive data without the need to formally install UHD.


Throughout the semester, we learned about wireless modulation and demodulation techniques, culminating in a final "mega" lab where we transmitted and received OFDM data with the Ettus B210 USRP. 
# meme
----
when in doubt, buy $3600 of software defined radios, 2 gpu accelerated sbcs, a 3d printer, then learn Rust, systems programming, linear algebra, digital signal processing, and **finally** rewrite the entire WiFi stack to stream videos from one side of the room to the other.

🤣


## Port Progress
---
[ ] Preamble (frequency correction)
[ ] Prefix ()
[ ] Guardbands 
