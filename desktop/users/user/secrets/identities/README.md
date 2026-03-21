# How to add new secret identity

## Machine Key

1. Generate new SSH Key

```shell
ssh-keygen -t ed25519 -f ~/.ssh/id_machine
cat ~/.ssh/id_machine.pub
cp ~/.ssh/id_machine.pub $(hostname).ssh.pub
```

## 1Password

1. Create or use an existing SSH key in 1Password
1. Generate age identity file:

```shell
age-plugin-op --generate "op://Personal/<item-name>/private_key" -o <username>-1password.age
```

3. Extract public key from the generated file:

```shell
grep "Recipient:" <username>-1password.age | awk '{print $3}' > <username>-1password.age.pub
```

## YubiKey

TBD;
