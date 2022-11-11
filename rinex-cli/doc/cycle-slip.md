Cycle Slip analysis & cancellation
==================================

Cycle slip detection and cancellation is a huge topic
in GNSS data processing.  
For advanced computations, it is most often a prerequisites.
That means such operations are not feasible or will not return
correct results if cycle slips were not cancelled prior to moving forward.

## Definitions

Let's consider Epoch $k$.  
A phase observation against carrier signal $L_i$ is defined as

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

And a cycle slip is as an abnormal phase step $|\Delta \Phi_{Li}(k)| > \epsilon$ between successive epochs
$\Delta \Phi_{Li}(k) = \Phi_{Li}(k) - \Phi_{Li}(k-1)$ 

## Possible cycle slips

Now all epochs in Observation RINEX come with a basic Cycle Slip indicator.  
They emphasize possible cycle slips at epoch $k$. Such epochs are emphasized by a Red circles, 
when Observation RINEX analysis is requested:

```bash
rinex-cli \
    --fp ../../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.rnx
	--retain-sv R18
```

Cycle slip determination
=========================

It is important to understand the previous information is not garanteed and simply an indicator.  
False positives happen due to simplistic algorithm in the receivers.  
False negatives happen due to lack of receiver capacity.  
Therefore, cycle slip determination algorithms are used to verify previous indications.

Several methods exist and this library uses 3 of them, depending on
the provided Observation Data.

In any case, this library is not limitated to $L_1$ and $L_2$ carriers, 
and is smart enough to form all possible combinations and scale them properly ( $\lambda_i$ ). 

## Doppler data [D]

Doppler measurements evaluate the variation rate of carrier phase
and are immune to cycle slips.

If doppler data exist for a given carrier signal $D(L_i)$ we
have a phase variation estimator

$$\Delta \Phi_{Li}(k) = \frac{ -d_k}{2} \left(D_{Li)(k) + D_{Li}(k-1) \right) $$

## Multi band / Modern context [GF]

Multi band context are the most "algorithm" friendly, 
as the expense of RINEX data complexity.

We form the geometry-free combinations easily:

$$\lambda_{Li} \Delta \Phi_{Li} - \lambda_{Lj} \Delta \Phi_{Lj} = \lambda_{Li} \left( \Delta N_{Li} + \Delta e_{Li} \right) - \lambda_{Lj} \left( \Delta N_{Lj} - \Delta e_{Lj} \right) + \Delta M_{Li} + \Delta M_{Lj} - \Delta I_{Li} \frac{\lambda^2_{Li} - \lambda^2_{Lj}}{\lambda^2_{Li}} $$

now let's rework the previous equation to emphasize $\Delta N_{Li} -  \Delta N_{Lj}$
the phase ambiguities difference two different carrier signals.


## Summary

Cycle slip determination is possible in all scenarios.  

- [D] is prefered due to its simplicity
- [GF] is the fallback method for modern contexts when Doppler shifts are missing.  
- [HOD] is the fallback method for basic contexts when Dopplers shifts are missing,
at the expense of a parametrization complexity
