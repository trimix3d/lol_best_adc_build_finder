# LoL best ADC build finder

A command line program that automatically finds the best combinations of items for ADCs in League of Legends.

Upon running the program, you will be prompted to choose the champion for which you want to find the best builds. It will generate multiple combinations of items for this champion and test each one by simulating fights against a target dummy (taking into account champions/items effects, ...). The combinations that gives the best results are then shown to you.

If needed, you can customize some settings related to the builds generation and the fights simulations (target tankiness, fight duration, force a specified item in the generated builds, and more).

Displaying the list of generated builds takes some space and can look weird if your window is not wide enough. So i recommend using fullscreen or widening the default window size.

I tried to make the interface as comprehensive as possible and if i've done my work correctly, you should be able to use it by yourself without further explanations. If at any point you are lost in the interface, type `help` to show help info on the current context. You can also navigate to the previous menu by typing `back` and go to the home page at any point by typing `home`. Type `exit` to exit the program.

I tried to get champions and items stats automatically but Riot's data dragon is ass (often has wrong values in it) so I entered them manually from the [LoL Wiki](https://wiki.leagueoflegends.com/en-us/) (god bless them) and will update them at each patch. For simplicity, the version number of each release is the same as the corresponding League of Legends patch number. If you have a reliable solution to automatically retrieve champions and items stats and put it in constants in the code, please let me know (I will be working to automate this in the near future).

This is a project i do during my free time. I try to document the code to the maximum, but it gets tedious (>﹏<). Also, no guarantees that i will update it forever.


# How to run

You have 2 options:

### 1) Download the lastest release

Go to the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) and download the executable (compatible only with windows on x86_64 CPUs), then run it.

### 2) Compile from source

If your OS/CPU architecture is not included in the release or if you want to compile the program yourself:

1. You need [Rust](https://www.rust-lang.org/tools/install) installed.
2. Download and extract the source code of the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) / git clone the repository.
3. Go in the directory containing the source code (on the same level as the `src` folder and `cargo.toml` file) and build the executable with the command ```cargo build --release```. The executable will be located in a newly created folder `target\release\`.


# How it works in more details

The project is separated in different modules:
- `game_data`: provides functions to manage champions, simulate fights against a target dummy and record the results.
- `build_optimizer`: generate the best combinations of items by using `game_data` to simulate them.
- `builds_analyzer`: tools for analyzing and displaying the output of `build_optimizer`. In the future i have plans to expand this module (making a tier list of differents champions based on their best builds performance?, ...).
- `cli`: command line interface to let the user interact with all of this.

Generating every possible combinations of n-items gives an absurd number of builds to try and this is impossible to process in reasonable time. That's why in `build_optimizer` I use another approach, based on the assumption that a good build made of n-items must also be a good build at n-1 items, and so on. This allows to drastically reduce the number of combinations because build options can now be explored like a tree and only the best "branches" are kept before choosing the next item, and so on.

After selecting a champion, the builds generation process works as the following, starting with a list containing one empty build at the beginning:

1. Select the first free slot (first slot for an empty build, second slot for a build with 1 item, etc).

2. For every build in the list, create copies with 1 additionnal item (from a predetermined pool of items for the champion) at the free slot for each.
    For exemple, if the current list of builds is:
    ```
    [{kraken_slayer}, //build containing 1 item: kraken_slayer
     {statikk_shyv }] //build containing 1 item: statikk_shyv
    ```
    And the items in the pool are `bloodthirster` and `infinity_edge`.
    The list of builds will become:
    ```
    [{kraken_slayer, bloodthirster},
     {kraken_slayer, infinity_edge},
     {statikk_shyv , bloodthirster},
     {statikk_shyv , infinity_edge}]
    ```

2. Simulate a fight with every build in the list and save the corresponding results.

    The results saved are:
    - price of the build
    - dps on the target
    - tankiness of the build (including heals and shields)
    - average effective move speed during the simulation (this is just `units_travelled/sim_duration`, so for exemple, dashes count as an increase in effective move speed)
    - some other stuff (special items utility, etc). A single score number is also calculated for each build from the price, dps, tankiness and average move speed.

3. Filter the builds from the list to keep only the better ones (within a certain configurable margin).

    The filtering is made of two parts:

    1. Keep builds that have a score within a predefined margin of the best score found.
    2. Keep builds that are part of the [pareto front](https://en.wikipedia.org/wiki/Pareto_front), the quantities to optimize being the build price, dps, tankiness, average move speed and utility of the build.

4. Repeat the process, building on top of the current build list until reaching the requested number of items.

I have simplified some things so it's easier to get the global picture. You can see how it works in detail in the code.


# About the results

It gives good results overall, even if in some cases you need to play around and fine tune the settings to avoid getting questionable builds.
It is really good at finding builds that gives the best pure damage output, a bit less good at finding builds with more utility that may be better in practice. That's why you need to analyze to results with common sense and experiment a bit.

If you have ideas about improving the program, feel free to share them :)
You can add me "trimix3d" on discord if you want to discuss about the project.