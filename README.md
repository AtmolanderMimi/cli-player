# CLI-Player

cli-terminal is a command line tool build with Rust that allows to play ascii representation of videos downloaded from youtube and videos from your computer in the terminal

It also doubles as a youtube downloader with the use of [rustube](https://github.com/DzenanJupic/rustube). (Go check it out!)

**REQUIRES FFMPEG TO BE INSTALLED FOR AUDIO**

Started: 2023-06-30


## Usage

Simply go into the cli-player directory and run the cli-player command in your terminal:

```text
~/Downloads/cli-player$ ./cli-player -q https://www.youtube.com/watch?v=dQw4w9WgXcQ 
```


### Arguments:

| short | long            | description                                     | default |
| ----- | --------------- | ----------------------------------------------- | ------- |
| `-q`  | `--query`       | The url or path to use when searching the video |         |
| `-p`  | `--pallet`      | Pallet of characters (from pallet file)         | ascii   |
| `-w`  | `--width`       | Number of characters in width                   | 100     |
| `-f`  | `--frame-limit` | Limits the frame rate (0 for native)            | 15      |
| `-v`  | `--volume`      | Sets the volume (can be over 1.0)               | 1.0     |
|       | `--preprocess`  | Preprocesses the frames                         |         |
|       | `--no-color`    | Disables the use of color                       |         |

**NOTE:**
* The height of the ascii representation of the video is relative to the width
* `--query` is for both youtube urls and system paths, the program will automatically figure out what it is
* `--preprocess` is almost useless, it takes up alot more RAM, so unless you somehow have alot of RAM, but very poor processing power don't use it
* Color may not work if your terminal does not support True Color


### Character Pallets:

Here are the ones already available:

| Name         | chracters                                                                                    |
| ------------ | -------------------------------------------------------------------------------------------- |
| braille-6    | `⠿⠽⠳⠪⠡⠄ `                                                                                   |
| braille-8    | `⣿⣻⣫⢭⢕⡡⢁⡀ `                                                                                 |
| ascii        | `@&%QWNM0gB$#DR8mHXKAUbGOpV4d9h6PkqwSE2]ayjxY5Zoen[ult13If}C{iF|(7J)vTLs?z/*cr!+<>;=^,_:'-.` |
| flat         | `█`                                                                                          |

You can add your own by editing the `character-pallets.txt` file 


## Known Limitations

* If there is too much time between frames the program will crash to avoid desync with the audio. If this appends lower the frame limitor
* The video downloading is very slow (about 1min of downloading for 1.5min of video)
* It was only tested on Linux. The main functionality should work, but there is no confirmation
* The program uses FFMPEG only for spliting the audio from the video, it should not be needed
* If your terminal font is not mono-spaced the effect will not work