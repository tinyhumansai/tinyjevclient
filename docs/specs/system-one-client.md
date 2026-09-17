# System One client

## Contract

The client mirrors `POST /v1/systemone`: state is a string, object, or array;
questions are a nonempty map of Choice, Score, and Noul values; answers return
under the same ids. Choice accepts 2–255 options, Score accepts 2–10 concrete
levels, and Noul may describe its true and false criteria.

The client validates request bounds before transport and validates response ids,
answer types, probability ranges and sums, selected maxima, Score legends, and
weighted Score values before returning. Score consistency allows two hundredths
for provider display rounding while probability sums retain strict tolerance.
Typed output is an interface guarantee,
not a truth guarantee; applications evaluate accuracy and thresholds on their
own data.

## Failure policy

Authentication and request errors are terminal. Transport failures, timeouts,
rate limits, overload, and server errors use a bounded caller-visible retry
policy. No retry is unbounded, and success or failure reports every attempt and
the full elapsed time. HTTP is allowed only for literal loopback addresses;
every remote endpoint requires HTTPS.

Credentials never appear in `Debug`, error messages, or retained response
bodies. Application state and provider bodies are not logged by the crate.
