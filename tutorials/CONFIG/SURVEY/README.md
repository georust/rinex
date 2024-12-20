Static surveying
================

Configuration scripts, load any of these with `ppp -c`.  

These scripts use high interpolation orders, which is compatible with long periods of signal observations.  
Reduce those when working with shorter observation periods. 

Any omitted field is in default state.   
Since default state is to compensate for any physical phenomena, we compensate for all of them in the provided setup.  

Here is an example to disable `relativistic_path_range` compensation:

```json
{
    "method": "CPP",
    "timescale": "GPST",
    "interp_order": 17,
    "min_sv_elevation": 10.0,
    "solver": {
        "filter": "Kalman",
        "gdop_threshold": 10.0
    },
    "modeling": {
        "relativistic_path_range": false
    } 
}
```
