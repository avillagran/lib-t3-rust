"""Run via android-sandbox; fake orchestration has no host or Android subprocesses."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parent.parent


class SandboxTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if os.environ.get('ANDROID_SANDBOX') != '1' or not Path('/run/.containerenv').is_file():
            raise SystemExit('These checks require android-sandbox')

    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix='android-sandbox-proof-'))
        self.project = self.root / 'project'
        self.project.mkdir()
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.sdk = self.root / 'sdk with spaces'
        self.sdk.mkdir()
        self.jdk = self.root / 'jdk'
        (self.jdk / 'bin').mkdir(parents=True)
        (self.jdk / 'bin/java').write_text('#!/bin/sh\nexit 97\n')
        (self.jdk / 'bin/java').chmod(0o700)
        fake = self.bin / 'podman'
        fake.write_text('''#!/usr/bin/python3
import json,os,sys
args=sys.argv[1:]
if args and args[0]=='--remote=false':args=args[1:]
if args[0]=='info': print(os.environ.get('MOCK_ROOTLESS','true'))
elif args[0]=='run':
 with open(os.environ['MOCK_ARGUMENTS'],'w') as output:json.dump(args,output)
else:raise SystemExit('unexpected fake orchestration operation')
''')
        fake.chmod(0o700)
        self.output = self.root / 'arguments.json'
        self.state = self.root / 'private cache'
        self.env = dict(os.environ, PATH=str(self.bin)+':'+os.environ['PATH'],
                        ANDROID_SANDBOX_SDK=str(self.sdk), ANDROID_SANDBOX_JDK=str(self.jdk),
                        ANDROID_SANDBOX_HOME=str(self.state), MOCK_ARGUMENTS=str(self.output),
                        SECRET_NOT_FOR_CONTAINER='must-not-be-forwarded')
        for key in ['GIT_DIR', 'GIT_WORK_TREE']:
            self.env.pop(key, None)

    def launch(self, *, cwd=None, env=None, command='printf "$HOME"'):
        return subprocess.run([str(SOURCE/'tools/android-sandbox'),'--sh',command],
                              cwd=cwd or self.project, env=env or self.env,
                              capture_output=True,text=True)

    def test_scoped_readonly_toolchains_private_cache_and_literal_command(self):
        result = self.launch(command='printf "$HOME"; echo literal')
        self.assertEqual(result.returncode,0,result.stderr)
        args = json.loads(self.output.read_text())
        self.assertIn('--read-only',args)
        self.assertIn('--network=none',args)
        self.assertIn(str(self.project)+':'+str(self.project)+':rw',args)
        self.assertIn(str(self.sdk)+':'+str(self.sdk)+':ro',args)
        self.assertIn(str(self.jdk)+':/opt/jdk:ro',args)
        self.assertIn(str(self.state/'home')+':/sandbox/home:rw',args)
        self.assertIn('HOME=/sandbox/home',args)
        self.assertNotIn('must-not-be-forwarded',json.dumps(args))
        self.assertEqual(args[-3:],['bash','-c','printf "$HOME"; echo literal'])
        self.assertEqual((self.state/'home').stat().st_mode & 0o777,0o700)

    def test_rootful_or_broad_cache_admission_precedes_fixture_creation(self):
        for env in [dict(self.env,MOCK_ROOTLESS='false'),dict(self.env,ANDROID_SANDBOX_HOME='/')]:
            result = self.launch(env=env)
            self.assertNotEqual(result.returncode,0)
            self.assertFalse(self.output.exists())
            self.assertFalse(self.state.exists())
        result = self.launch(cwd=Path('/'))
        self.assertNotEqual(result.returncode,0)
        self.assertFalse(self.output.exists())
        self.assertFalse(self.state.exists())

    def test_android_entrypoints_refuse_before_compilation_or_jvm_execution(self):
        env = dict(self.env)
        env.pop('ANDROID_SANDBOX',None)
        for name in ['build-android','compile-android-kotlin','package-android','test-jni','test-kotlin-jni']:
            result=subprocess.run([str(SOURCE/'tools'/name)],cwd=self.project,env=env,capture_output=True,text=True)
            self.assertEqual(result.returncode,2,(name,result.stdout,result.stderr))
            self.assertIn('require android-sandbox',result.stderr)
        self.assertFalse((self.project/'target').exists())

    def test_workflows_wrap_every_project_build_or_runtime_entry(self):
        commands=['cargo test','tools/test-c-abi','tools/test-jni','tools/test-kotlin-jni',
                  'tools/compile-android-kotlin','tools/build-android','tools/package-android']
        found=set()
        for file in (SOURCE/'.github/workflows').glob('*.yml'):
            for line in file.read_text().splitlines():
                if line.strip().startswith('- run:'):
                    for command in commands:
                        if command in line:
                            self.assertIn('android-sandbox --sh',line,(file,line))
                            found.add(command)
        self.assertEqual(found,set(commands))
        self.assertNotIn('rm -rf',(SOURCE/'tools/package-android').read_text())


if __name__=='__main__':
    unittest.main()
