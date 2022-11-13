GNSS signal combinations
========================

This page focuses on RINEX Observation Data.

Phase (PH) observations are precise but ambiguous (N in the following equations).  
Pseudo Range (PR) observations are not precise but they are unambiguous.  
You will see this characteristic if you ever use this tool serie
to visualize PR observations against PH observations.

Both of them are subject to so called Cycle Slips [CS], 
which are discontinuities in phase measurements due to
temporary loss of lock on the receiver side.  
One source of CS is a local clock jump.

For advanced computations, it is most often a prerequisites.
That means such operations are not feasible or will not return
correct results if CSs were not cancelled prior to moving forward.

Cycle slips happen randomly, seperately accross receiver channels,
and they affect Phase/Pseudo Range measurements.   

Phase model 
===========

Phase observation at epoch $k$ against carrier signal $L_i$ is defined as

$$\Phi_{Li}(k) = \frac{1}{\lambda_{Li}} \left( \rho(k)  + T(k) + S(k) + M_{Li}(k) - \frac{\lambda^2_{Li}}{\lambda^2_{Lj}}I(k) + \frac{c}{\lambda_{Li}} \left( \tau_{r}(k) + \tau_{sv}(k) \right) \right) + e_{Li} + N_{Li} $$  

$$\lambda_{Li} \Phi_{Li}(k) = \rho(k)  + T(k) + S(k) + M_{Li}(k) - \frac{\lambda^2_{Li}}{\lambda^2_{Lj}}I(k) + c \left( \tau_{r}(k) + \tau_{sv}(k) \right) + \lambda_{Li} e_{Li} + \lambda_{Li} N_{Li} $$  

where we note $\lambda_{Li}$, the $L_i$ carrier wavelength,  
$c$ the speed of light,  
$\rho$ the distance between the receiver APC and the vehicule APC - to be referred to as the _geometric_ distance,    
$\tau_{sv}$ is the vehicule clock bias [s],   
$\tau_{r}(k)$ the receiver clock bias [s],   
$M_{Li}$ the multipath biases,   
$e_{Li}$ the carrier phase thermal noise

Phase observations have an $N_{Li}$ cycles ambiguity.
When a phase bump appears, N varies by a random number. 

The abnormal phase step $|\Delta \Phi_{Li}(k)| > \epsilon$ between successive epochs
$\Delta \Phi_{Li}(k) = \Phi_{Li}(k) - \Phi_{Li}(k-1)$ 

Pseudo Range model
==================

TODO

GF recombination
================

We define the Geometry Free [GF] recombination

$$\lambda_{Li} \Delta \Phi_{Li} - \lambda_{Lj} \Delta \Phi_{Lj} = \lambda_{Li} \left( \Delta N_{Li} + \Delta e_{Li} \right) - \lambda_{Lj} \left( \Delta N_{Lj} - \Delta e_{Lj} \right) + \Delta M_{Li} + \Delta M_{Lj} - \Delta I_{Li} \frac{\lambda^2_{Li} - \lambda^2_{Lj}}{\lambda^2_{Li}} $$

now let's rework the previous equation to emphasize $\Delta N_{Li} -  \Delta N_{Lj}$
the phase ambiguities difference two different carrier signals.

GF recombination is requested with `--gf` when analyzing Observation RINEX.  
`--gf` is processed seperately, it does not impact the remaining record analysis (raw data),
it will just create a new visualization, in the form of "gf.png".   
GF expresses fractions of $L_i - L_j$ delay.

For example, `ESBC00DNK_R_2022` sampled G21 at a 30 second sample rate
(low sample rate). Form the GF recombination for both Pseudo Range and Phase data,
over a linear section:

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx \
    --retain-sv G21 \
    -w "2020-06-25 00:30:00 2020-06-25 02:00:00" \
    --gf --plot
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gf.png">

The residuals over code measurement is much more noisy (spreadout) than the residual phase,
whici is already down to some millimeters.

Be careful when requesting `--gf` to set a time window where all
codes to be recombined have a point at epoch 0. Otheriwse, the current phase alignment 
is not smart enough to align properly and you get residuals that are totally off, by orders of magnitude.

GF and atmospheric delay
========================

GF cancels out geometric terms but frequency dependant terms remain.
Therefore, GF is very good atmospheric delay estimator.

If we focus the previous phase data (most accurate measurement):

```bash
rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx \
    --retain-sv G21 \
    --retain-obs L1C,L2W \
    -w "2020-06-25 00:30:00 2020-06-25 02:00:00" \
    --gf --plot
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_gf_zoom.png">

The long term variations, particularly visible in the phase residuals,
are due to atmospheric condition changes.  

GF as a CS detector
===================

When analyzing Observation RINEX, we saw that we emphasize _possible_ CSs
when plotting the phase data.

CS affect RX channels independantly, randomly and is unpredictable.  

For example, R12(L3) in `ESBDNK2020` is particularly affected.  
You can see that L2 and L1 are not affected, for this vehicule
In this file, 100% of GPS vehicules are sane, and 95% of Glonass signals too.


Let's plot all phase data for L3 and L1 to emphasize the apparently faulty
clock cycles, and request a recombination

```bash
./target/release/rinex-cli \
    --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
    --retain-sv R12 \
    --retain-obs L3Q,L1C,L2P \
    -w "2020-06-25 01:00:00 2020-06-25 01:30:00" \
    --gf \
    --plot
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_r21_l1l3_zoom.png">

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_r21_l1l3_gf.png">

Discontinuities in GF slopes indicate bad receiption conditions and CSs.  
You can see the residual spread if you compare L3 against L1, to L2 against L1,
the latter being in the previous millimetric order that we saw before.

MW recombination
================

MW for "" is another recombination,
request it with by replacing `--gf` by  `--mw` in the previous example.  

If we go back to previous CS emphasis, we see that
Phase jumps are stretched out and clearly emphasized.

```bash

```

In summary, `--mw` is there and has only an interest when
user is interested in locating and visualizing CSs or phase bumps.  
For atmospheric delay visualization, `--gf` is totally fine.

When we'll move to CS cancellation, MW is internally preferred for enhanced
sensitivity. 

Doppler and phase estimator
===========================

Doppler measurements evaluate the variation rate of carrier phase
and are immune to cycle slips.

If doppler data exist for a given carrier signal $D(L_i)$ we
have a phase variation estimator

$$\Delta \Phi_{Li}(k) = \frac{(k+1)-k}{2} \left(D_{Li}(k) + D_{Li}(k-1) \right) $$


## Possible cycle slips

Now all epochs in Observation RINEX come with a basic Cycle Slip indicator.  
They emphasize possible cycle slips at epoch $k$. Such epochs are emphasized by a black symbols
on the RINEX analysis.

We saw that when describing Observation Record analysis, when focusing 
on Glonass L3 from `ESBDNK`:

```bash
rinex-cli \
    --fp ../../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz \
	--retain-sv R18
```

CS detection
============

It is important to understand the previous information is not garanteed and simply an indicator.  
False positives happen due to simplistic algorithm in the receivers.  
False negatives happen due to lack of receiver capacity.  
Therefore, cycle slip determination algorithms are used to verify previous indications.

In any case, this library is not limitated to $L_1$ and $L_2$ carriers, 
and is smart enough to form all possible combinations and scale them properly ( $\lambda_i$ ). 

We form the geometry-free [GF] combinations easily:


## Multi band / Modern context [GF]

Multi band context are the most "algorithm" friendly, 
at the expense of RINEX data complexity.


## Summary

Cycle slip determination is possible in all scenarios.  

- [D] is prefered due to its simplicity
- [GF] is the fallback method for modern contexts when Doppler shifts are missing.  
- [HOD] is the fallback method for basic contexts when Dopplers shifts are missing,
at the expense of a parametrization complexity
