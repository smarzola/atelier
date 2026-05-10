import pathlib
import unittest


class ReleaseWorkflowTests(unittest.TestCase):
    def test_release_workflow_builds_expected_unix_binary_targets(self):
        workflow = pathlib.Path('.github/workflows/release.yml').read_text()
        for target in [
            'x86_64-unknown-linux-gnu',
            'x86_64-apple-darwin',
            'aarch64-apple-darwin',
        ]:
            self.assertIn(target, workflow)
        self.assertIn('cargo build --release --target ${{ matrix.target }} --bin atelier', workflow)
        self.assertIn('softprops/action-gh-release@v3', workflow)

    def test_release_workflow_does_not_build_windows(self):
        workflow = pathlib.Path('.github/workflows/release.yml').read_text().lower()
        self.assertNotIn('windows', workflow)
        self.assertNotIn('pc-windows', workflow)
        self.assertNotIn('.zip', workflow)
        self.assertNotIn('pwsh', workflow)


if __name__ == '__main__':
    unittest.main()
