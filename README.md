# LoL best ADC build finder
A command line program that can automatically find the best combinations of items for ADCs in League of Legends.

Upon running the program, you will be prompted to choose the champion for which you want to find the best builds. It will generate multiple combinations of items for this champion and test each one by simulating fights against a target dummy (taking into account champions/items effects, ...). The combinations that gives the best results are then shown to you.

If needed, you can customize some settings related to the builds generation and the fights simulations (target tankiness, fight duration, force a specified item in the generated builds, and more).

Displaying the list of generated builds takes some space and can look weird if your window is not wide enough. For this reason, i recommend using fullscreen or widening the default window size.

I tried make the interface as comprehensive as possible and if i've done my work correctly, you should be able to use it by yourself without further explanations. If at any point you are lost in the interface, type `help` to show help info on the current context. You can also navigate back to a menu by typing `back` and go to the home page at any point by typing `home`. Type `exit` to exit the program.

For simplicity, the version number of each release is the same as the corresponding League of Legends patch number.

# How to run
You have 2 options:

### 1) Download the lastest release
Go to the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) and download the executable (compatible only with windows on x86_64 CPUs), then run it.

### 2) Compile from source
If your OS/CPU architecture is not included in the release or if you want to compile the program yourself:
1. You need [Rust](https://www.rust-lang.org/tools/install) installed.
2. Download and extract the source code of the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) / git pull the repository.
3. Go in the directory containing the source code (on the same level as the `src` folder and `cargo.toml` file) and build the executable with the command ```cargo build --release```. The executable will be located in a newly created folder `target\release\`.

# How it works in more details
WIP

# About the results
I think it gives good results overall, even if in some cases you need to play around and fine tune the settings to avoid getting questionable builds.
It is really good at finding builds that gives the best pure damage output, a bit less good at finding builds with more utility that may be better in practice. That's why you need to analyze to results with common sense and experiment a bit.

If you have ideas about improving the program, feel free to share them :)