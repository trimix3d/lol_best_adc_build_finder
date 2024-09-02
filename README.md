# LoL best ADC build finder
A command line program that can automatically find the best items combinations for an ADC in LoL.

Upon running the program, you will be prompted to choose the champion for which you want to find the best builds. It will generate multiple combinations of items for this champion and test each one by simulating fights against a target dummy (taking into account champions/items effects, ...). The combinations that gives the best results are then shown to you.

If needed, you can customize some settings related to the builds generation and the fights simulations (target tankiness, fight duration, force a specified item in the generated builds, and more).

Displaying the list of generated builds takes some space and can look weird if your window is not wide enough. For this reason, i recommend using fullscreen or widening the default window size.

I tried make the interface as comprehensive as possible and if i've done my work correctly, you should be able to use it by yourself without further explanations. If at any point you are lost in the interface, type `help` to show help info on the current context. You can also navigate back between the menus by typing `back` and go to the home page at any point by typing `home`.

For simplicity, the version number of this program follows the patch number of League of Legends.

# How to run
You have 2 options:

### 1) Download the lastest release
Go to the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) and download the executable (compatible only with windows on x86_64 CPUs), then run it.

### 2) Compile from source
If your OS/CPU architecture is not included in the release or if you want to compile the program yourself:
1. You need [Rust](https://www.rust-lang.org/tools/install) installed.
2. Download and extract the source code of the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) / git pull the repository.
3. Go in the directory containing the source code (on the same level as the `src` folder and `cargo.toml` file) and build the executable with the command ```cargo build --release```. The executable will be located in a newly created folder `target\release\`.

# How it works
WIP