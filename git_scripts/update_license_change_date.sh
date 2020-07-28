#! /bin/sh
# 
# Updates the BSL license change date on merge to master
# To enable, symlink to .git/hooks/pre-commit

# Debug
#log_file=`git rev-parse --show-toplevel`/out.log
#echo "Running script: `date`" >> $log_file

branch_name=`git branch | cut -b 3-`
dest_file=`git rev-parse --show-toplevel`/LICENSE_CHANGE_DATE.txt

if [ $branch_name = "master" ]
then
  this_year=`date +%Y`
  change_years=3
  future_year=$(( $this_year + $change_years ))
  echo $future_year-`date +%m-%d` > $dest_file
  git add $dest_file
  # The file will be included in the commit even though it doesn't appear in the
  # list to the editor.
# Debug
#  echo "Git added $dest_file" >> $log_file
fi
