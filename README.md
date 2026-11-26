# msczmutilate
This program modifies a few XML values in a .mscz file to bypass the "This file was saved using a newer version of MuseScore Studio" error. That's all it does, it does not correct any actual file changes between versions. I've tested it going from musescore 5.0 down to 4.7.7.
## Usage
```
msczultimate <inputpath> <optionaloutputpath>

```
If an output path is not provided, it will use <inputfilename>_mutilated.mscz.
