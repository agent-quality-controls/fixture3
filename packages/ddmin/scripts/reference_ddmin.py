from __future__ import annotations


def members(mask: int, candidate_count: int) -> list[int]:
    return [index for index in range(candidate_count) if mask & (1 << index)]


def to_mask(values: list[int]) -> int:
    mask = 0
    for value in values:
        mask |= 1 << value
    return mask


def split_ranges(values: list[int], count: int) -> list[list[int]]:
    chunks: list[list[int]] = []
    start = 0
    for partition_index in range(count):
        remaining_len = len(values) - start
        remaining_partitions = count - partition_index
        size = -((-remaining_len) // remaining_partitions)
        chunks.append(values[start : start + size])
        start += size
    return chunks


def reference_ddmin(candidate_count: int, outcomes: str, initial_granularity: int) -> tuple[int, str]:
    full = (1 << candidate_count) - 1
    if outcomes[full] != "I":
        return full, "incomplete:BaselineNotInteresting"

    current = list(range(candidate_count))
    granularity = max(initial_granularity, 2)
    while True:
        if granularity > len(current):
            return to_mask(current), "complete"

        chunks = split_ranges(current, granularity)
        reduced = False
        for chunk in chunks:
            candidate = to_mask(chunk)
            if outcomes[candidate] == "I":
                current = chunk
                granularity = 2
                reduced = True
                break
        if reduced:
            continue

        for chunk in chunks:
            removed = set(chunk)
            complement = [item for item in current if item not in removed]
            candidate = to_mask(complement)
            if outcomes[candidate] == "I":
                current = complement
                granularity = max(granularity - 1, 2)
                reduced = True
                break
        if reduced:
            continue

        if granularity >= len(current):
            return to_mask(current), "complete"
        granularity = min(len(current), granularity * 2)


def is_one_minimal(outcomes: str, candidate_count: int, mask: int) -> bool:
    if outcomes[mask] != "I":
        return False
    for item in members(mask, candidate_count):
        if outcomes[mask & ~(1 << item)] == "I":
            return False
    return True
