# Usage

All tasks are performed using odra-cli tool. There are handy aliases made in justfile, to ease up the usage.
To run tasks against the network, make sure you pass appropriate values in env, you can set up .env file - see .env.sample.
Environment variable decides the network and private key to use, so it is important to set it up correctly!
Basic usage for running tasks on the network is:

```bash
just cli
```

If you want to test on nctl, you can run the local node:

```bash
just run-nctl
```

And run the tasks with:

```bash
just cli-on-nctl
```

When running on nctl, the env variables are automatically set for you.

## Testing

To set up the environment for testing, we deploy our own version of WCSPR, deploy base contracts and finally,
deploy sample tokens and create pairs for them.

To deploy wcspr:

```bash
just cli scenario DeployWcspr
```

Note that this will add the contract to the container configuration file at `resources` folder. The name of the
file depends on the network name.

To deploy base contracts, we use special `deploy` option:

```bash
just cli deploy
```

This will result in Router, Factory and PairFactory contracts being deployed and saved in container.

Finally, if needed, we can run one of the scenarios, to set up samples:

```bash
just cli scenario SetupSamples # Will deploy two Sample tokens and create pairs
```

## Production

Production setup assumes that Wrapped CSPR (WCSPR) is already deployed and available on the network.
First step is to add WCSPR to the factory container:

```bash
just cli scenario AddWCSPR --package-hash hash-123123123...
```

Last step is to deploy base contracts:

```bash
just cli deploy
```

## Scenarios

Other scenarios are wrappers on the contracts methods, you can use them to perform some tasks easier:

```bash
just cli scenario SetupPair # Provide contracts to create a pair with
just cli scenario SwapTokens
just cli scenario AddLiquidity
just cli scenario AddLiquidityCSPR
just cli scenario SwapTokens
```

## Calling contracts
You can call contracts directly, e.g. 

```bash
just cli contract Router factory_address
```

To see all available commands, use `--help` switch.

will display address of the factory connected to the router.