Cycle Slip analysis & cancellation
==================================

Cycle slip [CS] detection and cancellation is a huge topic
in GNSS data processing.  
CSs are discontinuities in phase measurements due to
temporary loss of lock on the receiver side.  
One source of CS is a local clock jump.

For advanced computations, it is most often a prerequisites.
That means such operations are not feasible or will not return
correct results if CSs were not cancelled prior to moving forward.

Cycle slips happen randomly, seperately accross receiver channels,
and they affect Phase/Pseudo Range measurements.   

We saw in the record analysis that we emphasize _possible_ cycle slips
when plotting raw phase data. For example, a few Glonass vehicles
are affected in `ESBDNK2020`:

```bash
./target/release/rinex-cli --fp test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx --retain-sv R21,R12 -w "2020-06-25 00:00:00 2020-06-25 12:00:00"  --plot
```

<img align="center" width="650" src="https://github.com/gwbres/rinex/blob/main/doc/plots/esbc00dnk_glo_cs_zoom.png">

Almost all epochs at the beginning of the day were affected for R12(L3) and one or two for R21(L2).  
All GPS vehicles are sane, 95% of Glonass vehicles are sane too.

## Definitions

Let's consider Epoch $k$.  
A phase observation against carrier signal $L_i$ is defined as

$$\Phi_{Li}(k) = \frac{1}{\lambda_{Li}} \left( \rho(k)  + T(k) + S(k) + M_{Li}(k) - \frac{\lambda^2_{Li}}{\lambda^2_{Lj}}I(k) + \frac{c}{\lambda_{Li}} \left( \tau_{r}(k) + \tau_{sv}(k) \right) \right) + e_{Li} + N_{Li} $$  

$$\lambda_{Li} \Phi_{Li}(k) = \rho(k)  + T(k) + S(k) + M_{Li}(k) - \frac{\lambda^2_{Li}}{\lambda^2_{Lj}}I(k) + c \left( \tau_{r}(k) + \tau_{sv}(k) \right) + \lambda_{Li} e_{Li} + \lambda_{Li} N_{Li} $$  

where we note $\lambda_{Li}$, the $L_i$ carrier wavelength,  
$c$ the speed of light,  
$\rho$ the distance between the receiver APC and the vehicle APC - to be referred to as the _geometric_ distance,    
$\tau_{sv}$ is the vehicle clock bias [s],   
$\tau_{r}(k)$ the receiver clock bias [s],   
$M_{Li}$ the multipath biases,   
$e_{Li}$ the carrier phase thermal noise

Phase observations have an $N_{Li}$ cycles ambiguity.
When a phase bump appears, N varies by a random number. 

The abnormal phase step $|\Delta \Phi_{Li}(k)| > \epsilon$ between successive epochs
$\Delta \Phi_{Li}(k) = \Phi_{Li}(k) - \Phi_{Li}(k-1)$ 

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

$$\lambda_{Li} \Delta \Phi_{Li} - \lambda_{Lj} \Delta \Phi_{Lj} = \lambda_{Li} \left( \Delta N_{Li} + \Delta e_{Li} \right) - \lambda_{Lj} \left( \Delta N_{Lj} - \Delta e_{Lj} \right) + \Delta M_{Li} + \Delta M_{Lj} - \Delta I_{Li} \frac{\lambda^2_{Li} - \lambda^2_{Lj}}{\lambda^2_{Li}} $$

now let's rework the previous equation to emphasize $\Delta N_{Li} -  \Delta N_{Lj}$
the phase ambiguities difference two different carrier signals.

GF: CS and atmospheric delay
============================

GF cancels out geometric terms but frequency dependant terms remain.
GF is therefore a good atmospheric delay estimator.

When Observation Data is provided, GF recombination is requested
with `--gf`. When visualized, GF is always rescaled and displayed
in fractions of carrier delay

```bash
```

Discontinuities in GF slopes indicate cycle slips.

If we go back to `ESBDNK2020` Glonass L3 and perform GF recombination
on that data:

```bash

```

GF has a 1 cycle of $\lambda_i$  detection threshold. 
It is easy to understand that micro cycle slips will go undetected with
this estimator.

```bash

```

## MW or streched GF

MW for "" is another recombination,
request it with by replacing `--gf` by  `--mw` in the previous example.  

As far as atmospheric effect visualization goes, GF is totally fine.  
Here we're interested in increasing the discontinuity threshold.
MW scales the wavelength differently, producing a "stretching"
effect if you compare it to GF directly.

If we go back to previous CS emphasis, we see that
Phase jumps are stretched out and clearly emphasized.

```bash

```

In summary, `--mw` is there and has only an interest when
user is interested in visualizing phase jumps himself.
If you're only interested in atmospheric variations, GF is totally fine.
On the other hand, we the tool is to perform detection and cancellation itself,
it is MW recombination that will be prefered, due to enhanced sensitivity.

Doppler and phase estimator
===========================

Doppler measurements evaluate the variation rate of carrier phase
and are immune to cycle slips.

If doppler data exist for a given carrier signal $D(L_i)$ we
have a phase variation estimator

$$\Delta \Phi_{Li}(k) = \frac{(k+1)-k}{2} \left(D_{Li}(k) + D_{Li}(k-1) \right) $$

## Multi band / Modern context [GF]

Multi band context are the most "algorithm" friendly, 
at the expense of RINEX data complexity.


## Summary

Cycle slip determination is possible in all scenarios.  

- [D] is prefered due to its simplicity
- [GF] is the fallback method for modern contexts when Doppler shifts are missing.  
- [HOD] is the fallback method for basic contexts when Dopplers shifts are missing,
at the expense of a parametrization complexity
