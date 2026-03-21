# Fork.dev

## Custom Commands

### Absorb staged changes from commit

```bash
${git} absorb --base ${sha}
```

### Absorb staged changes on '${ref}'

```bash
${git} absorb --base $(${git} for-each-ref --format='%(upstream:short)' ${ref:full})
```
