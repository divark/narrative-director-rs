# Narrative Director User Manual

## Summary
Narrative Director is an alternative Audio/Video Recording application tailored
for working on medium to large-sized projects. This tool aspires to keep editing
to a minimum with the capability of playing, recording and re-recording readings
in place at the paragraph level for some text piece, whether it's a script, or a
novel.

## Getting Started
Begin by acquiring a .txt file that you wish to read. If you want to have a
sample to see Narrative Director's capabilities, use *War and Peace* provided by
gutenberg.org [here.](https://www.gutenberg.org/files/2600/2600-0.txt)

### Opening a Project
Load your text file by navigating to the Menu Bar, then clicking Open. Once the
file picker dialog appears, navigate to the text file and select it, then click
the Open button.

Here, Narrative Director shows the first paragraph if this is the first time
opening this file, or the last seen paragraph from the previous session.

### Viewing Paragraphs
Narrative Director shows the contents of a text file in paragraphs, where
paragraphs consists of four sentences or less. A counter at the top of the
program shows the current paragraph number out of the total found.

In order to move to the next paragraph, click the Next button. At any point, 
to move to the previous paragraph, click the Previous button.

If you want to jump to a particular paragraph number, this can be achieved
either by clicking the paragraph counter, or navigating to the Menu Bar and
going to View, then selecting Go To. A prompt will appear asking for the
paragraph number. When you are content with the number entered, click the
OK button. This will now show the paragraph at the spot specified.

To adjust the size of the text, go to the Menu Bar, click View, then
Zoom In or Zoom Out. To change the font of the text, see 
[Changing Preferences](#changing-preferences).

### Recording a Paragraph Reading
When opening Narrative Director for the first time, it uses the default input
device as deemed by the Operating System.

In addition, Narrative Director places all recordings in a folder named after the
chosen text file, located in the Music folder for your Operating System.

To change either of these, see [Changing Preferences](#changing-preferences).

-----------

To record or re-record a reading for the current paragraph in view, click the 
Record button. Take note of the playback time updating the total time. Note that you
will not be able to navigate to other paragraphs while you record.

If you wish to pause the recording, press the Pause button. This causes the
playback time to stop counting the total recording time. To resume, click the
Record button again.

When you are done recording, press the Stop button. This makes the progress
label show the total time of this newly created reading. Playback of this reading
will be possible now.

### Playing back a Reading
When opening Narrative Director for the first time, by default, it uses the
default output device as deemed by the Operating System.

If you wish to change this, see [Changing Preferences](#changing-preferences).

-----------

To hear an existing reading, navigate to a paragraph that has been recorded.
Once you do so, the playback time updates the total time of the reading, and
the Play button activates. Dragging the progress bar changes the start time of 
the reading. When satisfied, click the Play button, where you will hear the
reading. Notice the playback time counting up the current time to the total
time of the recording.

If you want to pause the playback at any time, press the Pause button. This
causes the playback time to stop counting the current position in the recording.
To resume, click the Play button again.

The recording stops playing once it reaches the end time, resetting the progress
label's current progress back to the beginning, as well as the progress bar. To
stop playback any sooner, press the Stop button.

### Changing Preferences
To open Preferences, go to the Menu Bar, then go to Edit, and click Preferences.

#### General
Here, you have the following options:

- Project Directory: This is where Narrative Director saves project folders. By
default, this will be the Music directory.
- Font: You can change the text's appearance here, factoring the type and size.

#### Audio
##### Output
This is where you change the device used for playback.

##### Input
This is where you change the device used for recording. In addition, the device
can be fine tuned with the following options:

- Sample Rate: This captures the amount of information to record. Making this
value higher will increase the audio quality at the cost of larger file sizes.
- Channels: This represents the number of directional inputs to consider based
on the capabilities of the device. For example, 2 represents a stereo recording.
