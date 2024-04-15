start = data_map["start"]
contents = content.split("\n")
if start not in contents[0]:
    error_message = "Operation check starting condition failed"
else:
    for line in contents:
        line = line.lower()
        new_content += line + "\n"
        content = content[len(line) + 1 :]
        content_index += len(line)
    data_map["end"] = contents[-1]
