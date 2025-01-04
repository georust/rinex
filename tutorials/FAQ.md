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

V4 navigation RINEx provides regular updates of the ionosphere model.
So it allows better ionosphere bias compensation while navigating using a single signal.  
Using modern GNSS receivers, this most likely does not buy you anything, because you are better
off using a second carrier or second constellation to do so: this is always the better choice.

Navigation V4 allows new high level operations like Earth orientation modeling and accurate compensations,
and Constellation timescale analysis.

## PPP (Precise Navigation)

1. What is PPP ?

PPP is a precise navigation algorithm that implicitly involves:

1. physical cancellation of the ionospheric bias
2. high end clock products (not radio based)
3. high end orbital products (not radio based)

(1) means you need a somewhat modern GNSS receiver. (2) + (3) physically
imply post processing, normally 3 weeks after observations, for the time
of the high end products to be released by some production agency.

The `rinex-cli` toolbox has a `ppp` option which we prefer to name `Post Processed Navgation`
for several reasons. For one, its behavior is totaly dependent on the input data.
You can deploy `ppp` without high end products, this is radio navigation use case, often
times refered to as "brdc" nav. It permits all kinds of "ppp" scenarios:

- with short term high end products (for example +1week and not +3week).
Once again you control the outcome with your input fileset
- use high end clock products with or without high end orbital products:
you can deploy `ppp` with what you have at your disposal.
This will limit your results
- SP3 as clock product: sometimes SP3 producer will provide both information
at the same time. You can deploy `ppp` with what you have at your disposal
- True `ppp` normally implies a Clock product expressed in the same timescale
as your physical observations. We call this scenario `Ultra-PPP` in our summary.
`ppp` allows you to deploy in both cases.

The Navigation compliance section of QC Summary is where to look at to understand
what your input fileset has to offer.

2. I have a hard time solving PPP solutions

First verify that your input fileset is compliant with at least one navigation technique,
by rendering a QC summary. 

Then, unlock the application logs to figure out what the solver might complain about:

1. is it complaining at deploy time ?
2. is it complaining at each epoch ? in this case you might have
an invalid setup with respect to your data
3. depending on your input setup, deployment requirements may be hardened
for a few iterations. The solver will let you know. If you don't respect that, it is physically
impossible to hope for deployment

## RTK (Precise Differential Navigation)

1. Is the toolbox RTK compatible ?

Not 100 % currently. We have put a lot of effort to unlock
the basic requirements to RTK navigation, so the basis is there.
You can for example study your RTK context by generating an analysis report with `rinex-cli`

2. I have a hard time defining my RTK setup

RTK processing is slightly simpler, but the definition is more complex.
To define your RTK setup, you have to specify which site is a "rover" (roaming device)
and which one is reference. By default, in rinex-cli, everything is defined as "rover".
You have to manually specify which site may serve as reference. To do so, you have two options

- load all observations made at reference site with `--rtk-ref` instead of `--fp,-f`
- load all observations with `--fp,-f` and specify the name or identifier of your
reference site(s) with `--rtk-ref`.

Until both sites are defined (at least one of each kind), deployment will not be compatible
with RTK! check your logs!

To determine if your setup is correctly defined, you may have several options at your disposal.
The simple being to render a quick `--sum` report, because the summary will expose all sites
defined as "rovers", and all reference sites!

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

CGGTTS also implies static surveying with a predefined position. This has two implications

1. you must provide the predefined static position, either from your RINEx or manually
2. this is normally incompatible with "roaming" devices
and is purely a laboratory setup. Since the toolbox has no means to know your end application,
it is totally possible to deploy `--cggtts` with roaming devices: it is simply not the 
typical use case that this scenario intends.

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
