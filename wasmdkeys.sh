source .env
echo "Paste this into the prompt: $MNEMONIC_MAIN"
wasmd keys add main --keyring-backend=test --recover
echo "Good. Now paste this into the prompt: $MNEMONIC_VALIDATOR"
wasmd keys add validator --keyring-backend=test --recover
