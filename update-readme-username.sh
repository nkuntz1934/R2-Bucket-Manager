#!/bin/bash

# Script to update README with your GitHub username

if [ $# -eq 0 ]; then
    echo "Usage: ./update-readme-username.sh YOUR_GITHUB_USERNAME"
    echo "Example: ./update-readme-username.sh johndoe"
    exit 1
fi

USERNAME=$1

echo "Updating README.md with GitHub username: $USERNAME"

# Update all occurrences of 'yourusername' with the actual username
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/yourusername/$USERNAME/g" README.md
else
    # Linux
    sed -i "s/yourusername/$USERNAME/g" README.md
fi

echo "✓ README.md updated successfully!"
echo ""
echo "Next steps:"
echo "1. Review the changes: git diff README.md"
echo "2. Commit the changes: git add README.md && git commit -m 'Update GitHub username in README'"
echo "3. Push to GitHub: git push origin main"