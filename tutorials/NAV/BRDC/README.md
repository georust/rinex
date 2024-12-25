BRDC Navigation
===============

Broadcast radio navigation is based on Navigation RINEx.  
It is the most simplistic navigation scenario we can think of, in the sense
it only requires stacking two RINEx files.

It is also often times refered to as _real time surveying_ or _real time navigation_,
in the sense it is physically the same as navigating at the same time the radio
message is decoded. RINEx just happens to offer to create a _snapshot_ of the
radio signals and allows _post processing_.

In the real world, mostly professional field surveying applications, data
is collected on the field and saved as RINEx. When back to base, the post processing
can happen. That means that processing time and power consumption is not an issue.

As opposed to [PPP](../PPP) scenarios, the navigation solutions can be obtained right away.
This is also why it is often times called _real time surveying_, once the RINEx files exist.
While the PPP scenario requires to gather (usually download) high end laboratory products,
that are available at the time of navigation.

## Signals :radio:

The RINEx and RTK toolboxes support most modern signals and we strive to support all of them.  
As long as your RINEx is correctly encoded, we can deploy the algorithm. 

The Signal Strategy in use is set by the `SignalStrategy` in the RTK Configuration script.  
It defines which signal we're using in that session. The toolbox supports both simplistic
(single signal) navigation scenarios, which sometimes prove very convenient, for example
in cheaper contexts. It also supports more advanced scenarios, which is compatible with
modern GNSS receivers that can sample several carriers at once. Only such strategy is compatible
with precise navigation.

Now having said that, it means you can use this toolbox to perform very precise radio based navigation.
The signal strategy simply defines what we extract from the observation RINEx. It is totally not related
to what other files is present in the context. You can use signal strategies that are typically implied
by so called "PPP" navigation technique, in a radio based context, using this toolbox. This is depicted
in most our BRDC examples.

## Constellations :artificial_satellite:

The RINEx and RTK toolboxes are supported to work on all constellations. Currently, it has been validated
in Galileo, GPS and BeiDou. We're working to make all options available in a near future.

## Time frame

The RINEx and RTK toolboxes are not limited to a specific timeframe. They will consume all the signals forwarded.
This is particularly useful because it will allow us to perform very precise long term surveying.
For example, loading two 24h RINEx observations naturally gives 48h of observations to consume.

Some of our examples illustrate that aspect as well.

## Tutorials

- [GPS](./GPS) examples that solely rely on this constellation
- [Galileo](./Galileo) examples that solely rely on this constellation
- [BeiDou](./BeiDou) examples that solely rely on this constellation
- [2024-Android-JMF](./2024-Android-JMF) is a dataset that
J. M. Friedt (`femto-st.fr`) laboratory has shared with us,
and was sampled in an urban environment, using a smartphone running
the RINEx plugin available on Android.
- [2024-NYA-JMF](./2024-NYA-JMF) was sampled in the Artic
by J. M. Friedt (`femto-st` labs) while surveying on a glacier.

