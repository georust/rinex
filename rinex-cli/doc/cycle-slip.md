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

## Cycle slip determination

It is important to understand the previous information is not garanteed by default.  
I suppose some GNSS receivers are also not able to indicate possible cycle slips.   
A cycle slip determination algorithm needs to be employed to verify previous informations and determine true cycle slip events.

Several methods exist. This library can perform cycle slip 
determination for both Single Carrier data (old receiver or old RINEX data),
and multi carrier context (modern RINEX, modern receivers).

The multi carrier context is the easiest context.  
In our example, `ESBC00DNK` is a RINEX3 with multi band information.  
