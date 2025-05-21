# btsg-wavs

An authentication contract validated an BLS12_381 signature, specifically for validating actions to take from transactions broadcasted by AVS operators.


## TODO
<!-- - verification: assert message to verify has been signed by one of the registered operator AVS keys (registered during `OnAuthenticatorAdded`) -->
<!-- - verification: assert we are accurately parsing the location where we expect the wavs operator AVS public key, signature, and message that was signed from the transaction (specifially from `AuthenticationRequest`) -->
<!-- - msgs: implement contract owner & owner migration workflow (not need, workflow will be to unregister and reregister) -->
-- integration tests

## Forming Authentication Requests Msgs