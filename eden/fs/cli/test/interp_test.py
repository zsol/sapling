# Copyright (c) 2017-present, Facebook, Inc.
# All rights reserved.
#
# This source code is licensed under the BSD-style license found in the
# LICENSE file in the root directory of this source tree. An additional grant
# of patent rights can be found in the PATENTS file in the same directory.

from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals
from .. import configinterpolator
import configparser
import unittest


class InterpolatorTest(unittest.TestCase):
    def test_basic_subs(self):
        defaults = {'USER': 'wez', 'RECURSIVE': 'a${RECURSIVE}b'}
        parser = configparser.ConfigParser(
            interpolation=configinterpolator.EdenConfigInterpolator(defaults))
        parser.add_section('section')
        parser.set('section', 'user', '${USER}')
        parser.set('section', 'rec', '${RECURSIVE}')
        parser.set('section', 'simple', 'value')

        self.assertEqual('wez', parser.get('section', 'user'))
        self.assertEqual('value', parser.get('section', 'simple'))
        self.assertEqual('a${RECURSIVE}b', parser.get('section', 'rec'))

        actual = {}
        for section in parser.sections():
            actual[section] = dict(parser.items(section))

        expect = {
            'section': {
                'user': 'wez',
                'simple': 'value',
                'rec': 'a${RECURSIVE}b',
            }
        }
        self.assertEqual(expect, actual)
