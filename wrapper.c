#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "wrapper.h"

void print_status(Story story)
{
	printf("Status:%s\nContinue?: %s\nChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);
}

void string_test()
{
	string* test_str = new_string();
	write_string(test_str, "Hello!");

	char* read_buf = malloc(1);
	read_buf[0] = '\0';
	read_string(test_str, read_buf);
	printf("Test string: \'%s\'\n", read_buf);

	printf("Flushing string:\n");
	flush_string(test_str);
	printf("\n");

	free_string(test_str);
}

int main() 
{
	//string_test();

	printf("Initilizing:\n");
	Story story = NewStory(); 
	do{
		while(CanContinue(story))
		{
			printf("Stepping..\n");
			string* str = ContinueMaximally(story);
			flush_string(str);
			printf("\n");
		}

		if(ChoiceCount(story) > 0)
		{
			unsigned int choice = 0;
			print_status(story);
			printf("Chose choice: ");
			scanf("%u", &choice);
			ChooseChoiceIndex(story, choice);
		}

	} while(CanContinue(story) || ChoiceCount(story) > 0);

	return 0;
}
