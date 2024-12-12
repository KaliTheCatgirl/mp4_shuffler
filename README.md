# MP4 Shuffler
A small, cobbled-together datamoshing program that mutilates MP4 files by shuffling P-frames. Requires `ffmpeg` to be installed.

## Usage
To run the program, simply run `mp4_shuffler <input> <output>`. There will be two new files. One is a remuxed version of the original file (required for the datamosh to work effectively) and the other is the output file. The filename of the remuxed version will simply have `.mux.tmp` appended to the input file's name.

Before deleting the temporary file, it is recommended to validate the output of the file. This program is built to be run multiple times, as it program cannot validate the outputted MP4 files, so it may require multiple runs to produce a usable file. However, once the program has been run once, the temporary file can be reused. Simply use the temporary file as input and add `-p` to the program arguments.

To automatically remove the temporary file (not recommended), add `-r` to the program arguments.

To only shuffle past a certain point in the video, add the `-s` flag, followed by a number from 0 to 1 indicating a fraction. A value of 0 will shuffle the whole video, a value of 1 will shuffle nothing. Any value in between will start the shuffling process at that point. For example, specifying `-s 0.2` will keep the first 20% of the video intact, shuffling all other frames.
