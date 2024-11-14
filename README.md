# LoL best ADC build finder

A command line program that automatically finds the best combinations of items for ADCs in League of Legends.

Upon running the program, you will be prompted to choose the champion for which you want to find the best builds. It will generate multiple combinations of items for this champion and test each one by simulating fights against a target dummy (taking into account champions/items effects, ...). The combinations that gives the best results are then shown to you.

If needed, you can customize some settings related to the builds generation and the fights simulations (target tankiness, fight duration, force a specified item in the generated builds, and more).

Displaying the list of generated builds in the terminal takes some space and can look weird if your window is not wide enough. So i recommend widening the default window size or using fullscreen.

I tried to make the interface as comprehensive as possible and if i've done my work correctly, you should be able to use it by yourself without further explanations. If at any point you are lost in the interface, type `help` to show help info on the current context. You can also navigate to the previous menu by typing `back` and go to the home page at any point by typing `home`. Type `exit` to exit the program.

This is a project i do during my free time. I try to document the code to the maximum, but it gets tedious (>﹏<). Also, no guarantees that i will update it forever.


# How to run

You have 2 options:

### 1) Download the lastest release

Go to the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) and download the executable (compatible only with windows on x86_64 CPUs), then run it.

### 2) Compile from source

If your OS/CPU architecture is not included in the release or if you want to compile the program yourself:

1. You need [Rust](https://www.rust-lang.org/tools/install) installed.
2. Download and extract the source code of the lastest [release](https://github.com/trimix3d/lol_best_adc_build_finder/releases) / git clone the repository.
3. Navigate in the directory containing the source code (on the same level as the `src` folder and `cargo.toml` file) and build the executable with the command ```cargo build --release```. The executable will be located in a newly created folder `target\release\`.


# How it works in more details

The project is separated in different modules:
- `game_data`: provides functions to manage champions, simulate fights against a target dummy and record the results.
- `champion_optimizer`: finds the best build/runes for a champion by using the `game_data` module to simulate them.
- `builds_analyzer`: tools for analyzing and displaying the output of `champion_optimizer`. In the future i have plans to expand this module (making a tier list of differents champions based on their best builds performance?, ...).
- `cli`: command line interface to let the user interact with all of this.

Generating every possible combinations of n-items gives an absurd number of builds to try and this is impossible to process in reasonable time. That's why in `champion_optimizer` I use another approach, based on the assumption that a good build made of n items must also be a good build at n-1 items, and so on. This allows to drastically reduce the number of combinations because builds can now be explored like a tree where we only keep the best branches.

The builds generation process works as the following: after selecting a champion, it starts with a list containing one empty build at the beginning:

1. Select the first build of the current list. Loop through a predetermined pool of items available for the champion. For each of these items, add them to a copy of the selected build and store this copy in a new list. Repeat for every build of the list.

    For exemple, if the current list of builds is:
    ```
    [{kraken_slayer}, //build containing 1 item: kraken_slayer
     {statikk_shyv }] //build containing 1 item: statikk_shyv
    ```
    And the items available for the champion are `bloodthirster` and `infinity_edge`.
    The new list of builds will be:
    ```
    [{kraken_slayer, bloodthirster},
     {kraken_slayer, infinity_edge},
     {statikk_shyv , bloodthirster},
     {statikk_shyv , infinity_edge}]
    ```
    At the end of this step, the new list contains all the possible builds combination with one additionnal item to the builds of the old list. The old list is discarded.

2. Simulate a fight with every build in the new list and save the corresponding results.

    The results saved are:
    - price of the build
    - dps on the target
    - tankiness of the build (including heals and shields)
    - average effective move speed during the simulation (this is just `units_travelled/sim_duration`, so for exemple, dashes count as an increase in effective move speed)
    - some other stuff (special items utility, etc).

3. Filter the builds from the new list to keep only the better ones (within a certain configurable margin).

    The filtering is made of two parts:

    1. Keep builds that have a score (based on its price, dps, tankiness and mobility) within a predefined margin of the best score found.
    2. Keep builds that are part of the [pareto front](https://en.wikipedia.org/wiki/Pareto_front), the quantities to optimize being the build price, dps, tankiness, average move speed and utility of the build.

4. Repeat the process, building on top of each subsequent build list until reaching the requested number of items.

This description has been simplified to get the global picture more easily. Feel free to see how it works in detail in the code.


# About the results

It gives solid results overall. With the default settings, it finds most of the time the correct, well established builds you can see on sites like [LoLalytics](https://lolalytics.com/) or [op.gg](https://www.op.gg/) for most of the champions.
But the power of this tool is not in finding those "common" builds, it's in its ability to tweak the settings to explore builds that perform best in a specific use case / find builds that have not yet been discovered when there isn't enought data gathered after a patch.
Depending on your settings, you may get questionable builds though, so always analyze the results with common sense. This tool is made to be experimented with!


# Todo list

- [x] add every relevant S14 ADC items - DONE
- [ ] add every ADC - in progress
- [ ] add every runes keystones - in progress
- [ ] retrieve champions and items data automatically from community dragon (instead of updating values manually each patch) - to do
- [ ] make a GUI - to do

If you have ideas about improving the program, I'm open to suggestions and pull requests :)
Feel free to add `trimix3d` on discord if you want to discuss about the project.