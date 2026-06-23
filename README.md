# zr_sq
Minimalist-ish sequencer that generates audio based on a custom plaintext file format.

Unfortunately I got this started by vibecoding with deepseek, but I'm planning on that being the only AI assistance for this project as I become more familiar with rust.

# To-Do (feature creep) : 
- Implement BPM alignment for rest 
- Implement basic parsing of notes - translate A4 -> 440hz
    - Have a header option for the "tuning ceneter" note - A4 by default 
    - Have a header option for just tuning - perfect ratios, vs logorithmic cents scale

- Clean up messy AI code - condense the file parsing matcher into one function 
    - Change "waveform" into something more general like "function"
- Add main loop for live updating of file & playback
- Add ability to create loops & define musical "functions" within a file, that will play in parallel to the rest of the file
- Stereo sound






# Usage / installation
Make sure you have a way to listen to ALSA outputs properly set up, I wasted a good few hours debugging this program at first just to find that I didn't have `pipewire-alsa` installed on my arch linux system.




# Song file format
Look at `example.music` for more, but to start you can type 
```saw 440.0 0.1 0.8``` to play a saw wave at 440hz, for 0.1 seconds, at 0.8 volume (max of 1.0, min of 0)

and rest 







