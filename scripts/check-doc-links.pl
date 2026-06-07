#!/usr/bin/env perl
use strict;
use warnings;
use File::Basename qw(dirname);
use File::Find;
use File::Spec;

my @files;
find(
    sub {
        return unless -f $_;
        return unless /\.md\z/;
        push @files, $File::Find::name;
    },
    "docs",
);
push @files, "README.md", "SECURITY.md";

my $failed = 0;

for my $file (@files) {
    open my $fh, "<", $file or die "open $file: $!";
    my $line_no = 0;
    while (my $line = <$fh>) {
        ++$line_no;
        while ($line =~ /\[[^\]]+\]\(([^)]+)\)/g) {
            my $target = $1;
            next if $target =~ m{\Ahttps?://};
            next if $target =~ m{\Amailto:};
            next if $target =~ m{\A#};
            next if $target =~ m{\A/};
            my $path = $target;
            $path =~ s/#.*\z//;
            next if $path eq "";
            my $resolved = File::Spec->canonpath(File::Spec->catfile(dirname($file), $path));
            if (!-e $resolved) {
                print STDERR "$file:$line_no: broken link target: $target\n";
                $failed = 1;
            }
        }
    }
}

exit $failed;
