.jobs[] | {id, short: .name[0:45], conclusion, failedStep: ([.steps[] | select(.conclusion=="failure") | .name] | join(", "))}
