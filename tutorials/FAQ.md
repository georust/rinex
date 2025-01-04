Frequently Asked Questions
==========================

## RINEx (Receiver Indepdent Ex-change format)

1. Is the toolbox V4 compatible ?

Yes. V4 revision only impacts navigation RINEX for the most part.
We support all new navigation updates that V4 offers.
To try it out, you can for example:
- render an analysis by loading RINEX V4 nav into rinex-cli
- use your RINEX V4 nav for ppp with rinex-cli

2. What is the main difference with V4 RINEx versus V3 RINEx ?

It only impacts high precision post processed navigation.
In V3, you are limited to one set of parameters per 24h time frame (basically one RINEx)
available daily - midnight to midnight. For example the ionosphere model is therefore
very limited and somewhat unrealistic. In V4 you get regular updates, just like you would
in the field using your RF antenna, so the model is regularly updated and re-ajusted. 
In short, NAV V4 is a closer representation of what navigation message decoding has to offer.

## RTK

1. Is the toolbox RTK compatible ?

Not 100 % currently. We have put a lot of effort to unlock
the basic requirements to RTK navigation, so the basis is there.
You can for example study your RTK context by generating an analysis report with `rinex-cli`

## CGGTTS

1. What is CGGTTS ?

CGGTTS is a file format specified by BIPM which allows remote clock comparison
by means of common satellites sighted on both ends.
Refer to the related library, Wikipedia and the BIPM for more information.

2. Is CGGTTS simply "PPP" or how do they differ ?

CGGTTS and PPP are two different things. CGGTTS solutions production is the outcome of PPP
navigation, simply slightly modified and presented in a very specific way - as per the BIPM specifications.
The slight differences being

- the measurements are collected the averaged, according to a periodic table
- CGGTTS scheduling is incompatible with unsteady sampling (observation period)

3. Why is it so long to obtain CGGTTS solutions ?

For the simple reason that solutions are produced for any SV in sight. Simply
reduce the number of SV and/or constellations in your input (with `-P` for example, or using
different datasets) to reduce the number of solutions that we can resolve.

## IONEx

1. Does the toolbox support the IONEx format ?

Yes. All RINEx formats are currently supported (on the parsing side) and soon
on the formatting side. 

2. What does IONEx have to offer ?

Loading a IONEx along your other RINEx may serve many purposes.
Our toolbox allows IONEx analysis by itself, which serves Ionosphere status
analysis. The next releases of our toolbox may help synthesize IONEx from observation
RINEx as well. Finally, in the context of PPP navigation, IONEX might be useful when working
with V3 RINEx, because it can serve as a ionosphere model updater.

On the other side, you can more deeply analyze your Observation RINEx if you
have a IONEx for that day.
