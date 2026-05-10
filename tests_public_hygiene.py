import unittest
from pathlib import Path


class PublicHygieneScannerTests(unittest.TestCase):
    def test_scanner_does_not_contain_tracked_private_literal_denylist(self):
        scanner = Path("scripts/hygiene-scan.sh").read_text()

        self.assertIn(".atelier-local/hygiene-denylist.txt", scanner)
        self.assertIn("Do not put real personal identifiers", scanner)
        self.assertIn("generic_patterns", scanner)
        self.assertNotIn("chat-123", scanner)
        self.assertNotIn("topic-456", scanner)
        self.assertNotIn("/home/", scanner)


if __name__ == "__main__":
    unittest.main()
