# Solar-Battery System Simulation
This is a simple plot integrating the energy storage and usage in a simple battery-solar system. 

This simulation is reasonable for approximating the scale of a solar panel and battery for almost any steady load, especially when the load doesn't change with seasons. Mobile sensor stations, outdoor lights, or even rough approximations of an off-grid house are possible. Loads like community refrigerators are also possible but the load to keep a refrigerator cold will fluctuate with outdoor temperature, and is a feature to implement in the future.

The user interface lets you change most of the simulation parameters to see how your system will perform.

![User Interface](GUI.png?raw=true)

## Assumptions
* The load in the system is assumed to be constant with time. 
* Solar energy is approximated from the input latitude, producing a sinusoidal curve of power from sunrise to sunset.
* No losses in the battery and inverter are yet modeled. 
* All energy not being directly consumed by the load is stored in the battery. Any deficit is pulled from the battery.
